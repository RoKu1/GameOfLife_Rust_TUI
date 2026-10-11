use std::{
    error::Error,
    io::{self, Stdout},
    panic::{self, AssertUnwindSafe},
    time::{Duration, Instant},
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use game_of_life::grid::{Board, Pattern, PATTERNS};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell as TableCell, Paragraph, Row, Table},
    Frame, Terminal,
};

type AppTerminal = Terminal<CrosstermBackend<Stdout>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Screen {
    Setup,
    Workspace,
    Simulation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartSource {
    Workspace,
    Pattern,
    Random,
}

struct App {
    screen: Screen,
    source: StartSource,
    selected_pattern: usize,
    board: Board,
    seed: Board,
    seed_name: String,
    cursor: (usize, usize),
    generation: u64,
    running: bool,
    speed_ms: u64,
}

impl App {
    fn new() -> Self {
        let board = Board::game_board();
        Self {
            screen: Screen::Setup,
            source: StartSource::Workspace,
            selected_pattern: 0,
            board: board.clone(),
            seed: board,
            seed_name: "Workspace".to_string(),
            cursor: (50, 50),
            generation: 0,
            running: true,
            speed_ms: 120,
        }
    }

    fn current_pattern(&self) -> &'static Pattern {
        &PATTERNS[self.selected_pattern]
    }

    fn start(&mut self) {
        match self.source {
            StartSource::Workspace => {}
            StartSource::Pattern => self.board.load_pattern(self.current_pattern()),
            StartSource::Random => self.board.randomize(),
        }
        self.seed_name = match self.source {
            StartSource::Workspace => "Workspace".to_string(),
            StartSource::Pattern => self.current_pattern().name.to_string(),
            StartSource::Random => "Random".to_string(),
        };
        self.seed = self.board.clone();
        self.generation = 0;
        self.running = true;
        self.screen = Screen::Simulation;
    }

    fn step(&mut self) {
        self.board.step();
        self.generation = self.generation.saturating_add(1);
    }
}

struct TerminalGuard {
    terminal: AppTerminal,
}

impl TerminalGuard {
    fn new() -> Result<Self, Box<dyn Error>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = execute!(stdout, EnterAlternateScreen) {
            let _ = disable_raw_mode();
            return Err(Box::new(error));
        }
        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(error) => {
                let _ = execute!(io::stdout(), LeaveAlternateScreen);
                let _ = disable_raw_mode();
                return Err(Box::new(error));
            }
        };
        let mut guard = Self { terminal };
        guard.terminal.hide_cursor()?;
        Ok(guard)
    }

    fn restore(&mut self) -> io::Result<()> {
        let raw_mode = disable_raw_mode();
        let leave_screen = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let cursor = self.terminal.show_cursor();
        raw_mode.and(leave_screen).and(cursor)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = self.restore();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = TerminalGuard::new()?;
    let result = panic::catch_unwind(AssertUnwindSafe(|| run_app(&mut terminal.terminal)));
    terminal.restore()?;
    match result {
        Ok(result) => result,
        Err(payload) => panic::resume_unwind(payload),
    }
}

fn run_app(terminal: &mut AppTerminal) -> Result<(), Box<dyn Error>> {
    let mut app = App::new();
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| draw(frame, &app))?;
        let timeout = if app.screen == Screen::Simulation && app.running {
            Duration::from_millis(app.speed_ms).saturating_sub(last_tick.elapsed())
        } else {
            Duration::from_millis(100)
        };
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release {
                    continue;
                }
                let was_running = app.running;
                let was_simulating = app.screen == Screen::Simulation;
                if handle_key(&mut app, key.code) {
                    return Ok(());
                }
                if app.screen == Screen::Simulation
                    && (!was_simulating || (!was_running && app.running))
                {
                    last_tick = Instant::now();
                }
            }
        }
        if app.screen == Screen::Simulation
            && app.running
            && last_tick.elapsed() >= Duration::from_millis(app.speed_ms)
        {
            app.step();
            last_tick = Instant::now();
        }
    }
}

fn handle_key(app: &mut App, key: KeyCode) -> bool {
    match app.screen {
        Screen::Setup => match key {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Up | KeyCode::Char('k') => {
                app.source = match app.source {
                    StartSource::Workspace => StartSource::Random,
                    StartSource::Pattern => StartSource::Workspace,
                    StartSource::Random => StartSource::Pattern,
                };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.source = match app.source {
                    StartSource::Workspace => StartSource::Pattern,
                    StartSource::Pattern => StartSource::Random,
                    StartSource::Random => StartSource::Workspace,
                };
            }
            KeyCode::Left | KeyCode::Char('h') if app.source == StartSource::Pattern => {
                app.selected_pattern = if app.selected_pattern == 0 {
                    PATTERNS.len() - 1
                } else {
                    app.selected_pattern - 1
                };
            }
            KeyCode::Right | KeyCode::Char('l') if app.source == StartSource::Pattern => {
                app.selected_pattern = (app.selected_pattern + 1) % PATTERNS.len();
            }
            KeyCode::Char('e') if app.source == StartSource::Workspace => {
                app.screen = Screen::Workspace
            }
            KeyCode::Enter | KeyCode::Char(' ') => app.start(),
            _ => {}
        },
        Screen::Workspace => match key {
            KeyCode::Esc => app.screen = Screen::Setup,
            KeyCode::Enter => {
                app.source = StartSource::Workspace;
                app.start();
            }
            KeyCode::Char(' ') | KeyCode::Char('x') => app.board.toggle(app.cursor.0, app.cursor.1),
            KeyCode::Char('c') => app.board.clear(),
            KeyCode::Left | KeyCode::Char('h') => app.cursor.0 = app.cursor.0.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => {
                app.cursor.0 = (app.cursor.0 + 1).min(app.board.width() - 1)
            }
            KeyCode::Up | KeyCode::Char('k') => app.cursor.1 = app.cursor.1.saturating_sub(1),
            KeyCode::Down | KeyCode::Char('j') => {
                app.cursor.1 = (app.cursor.1 + 1).min(app.board.height() - 1)
            }
            _ => {}
        },
        Screen::Simulation => match key {
            KeyCode::Char('q') => return true,
            KeyCode::Esc => app.screen = Screen::Setup,
            KeyCode::Char(' ') | KeyCode::Char('p') => app.running = !app.running,
            KeyCode::Char('s') => {
                app.running = false;
                app.step();
            }
            KeyCode::Char('r') => {
                app.board = app.seed.clone();
                app.generation = 0;
                app.running = false;
            }
            KeyCode::Char('e') => {
                app.running = false;
                app.source = StartSource::Workspace;
                app.screen = Screen::Workspace;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                app.speed_ms = app.speed_ms.saturating_sub(20).max(20)
            }
            KeyCode::Char('-') => app.speed_ms = (app.speed_ms + 20).min(1000),
            _ => {}
        },
    }
    false
}

fn draw(frame: &mut Frame<'_>, app: &App) {
    if frame.area().width < 80 || frame.area().height < 13 {
        let message = Paragraph::new("Terminal too small\nResize to at least 80 x 13")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        frame.render_widget(message, frame.area());
        return;
    }
    match app.screen {
        Screen::Setup => draw_setup(frame, app),
        Screen::Workspace => draw_workspace(frame, app),
        Screen::Simulation => draw_simulation(frame, app),
    }
}

fn draw_setup(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(7),
            Constraint::Length(3),
        ])
        .split(area);
    let title = Paragraph::new("GAME OF LIFE  ·  Choose a starting state")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    let content = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
        .split(chunks[1]);
    let sources = [
        (
            StartSource::Workspace,
            "Current workspace",
            "Edit a blank board or continue with its current state",
        ),
        (
            StartSource::Pattern,
            "Pattern library",
            "Choose a curated classic pattern",
        ),
        (
            StartSource::Random,
            "Random board",
            "Fill the board with a random starting state",
        ),
    ];
    let mut rows = Vec::new();
    for (source, label, hint) in sources {
        let selected = source == app.source;
        rows.push(
            Row::new(vec![
                TableCell::from(if selected {
                    format!("> {label}")
                } else {
                    format!("  {label}")
                }),
                TableCell::from(hint),
            ])
            .style(if selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            }),
        );
    }
    let table = Table::new(
        rows,
        [Constraint::Percentage(50), Constraint::Percentage(50)],
    )
    .header(Row::new(vec!["START FROM", "DESCRIPTION"]).style(Style::default().fg(Color::Cyan)))
    .block(
        Block::default()
            .title("Starting state  ·  ↑/↓")
            .borders(Borders::ALL),
    );
    frame.render_widget(table, content[0]);

    let details = if app.source == StartSource::Pattern {
        let pattern = app.current_pattern();
        format!(
            "{}\n\n{}\n\nPattern {}/{}  ·  use ←/→ to browse",
            pattern.name,
            pattern.description,
            app.selected_pattern + 1,
            PATTERNS.len()
        )
    } else if app.source == StartSource::Workspace {
        format!(
            "Current workspace\n\n{} live cells\n\nPress E to edit the board before starting.",
            app.board.live_count()
        )
    } else {
        "Random board\n\nA fresh random state is generated every time you start.".to_string()
    };
    let info = Paragraph::new(details)
        .style(Style::default().fg(Color::White))
        .block(Block::default().title("Details").borders(Borders::ALL));
    frame.render_widget(info, content[1]);

    let help =
        Paragraph::new("↑/↓: source  ·  ←/→: pattern  ·  E: edit  ·  Enter: start  ·  Q: quit")
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

fn draw_workspace(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);
    let title = Paragraph::new(format!(
        "EDIT CURRENT WORKSPACE  ·  {} live cells",
        app.board.live_count()
    ))
    .alignment(Alignment::Center)
    .style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);
    draw_board(frame, app, chunks[1], true);
    let help = Paragraph::new(
        "Arrows/HJKL move  ·  Space/X toggle  ·  C clear  ·  Enter start  ·  Esc back",
    )
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

fn draw_simulation(frame: &mut Frame<'_>, app: &App) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(3),
            Constraint::Length(3),
        ])
        .split(area);
    let state = if app.running { "RUNNING" } else { "PAUSED" };
    let title = Paragraph::new(format!(
        "GAME OF LIFE  ·  Gen {}  ·  {} live  ·  {}  ·  {}ms",
        app.generation,
        app.board.live_count(),
        state,
        app.speed_ms
    ))
    .alignment(Alignment::Center)
    .style(
        Style::default()
            .fg(if app.running {
                Color::Green
            } else {
                Color::Yellow
            })
            .add_modifier(Modifier::BOLD),
    )
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);
    draw_board(frame, app, chunks[1], false);
    let help = Paragraph::new(
        "Space/P pause  ·  S step  ·  R reset  ·  E edit  ·  +/- speed  ·  Esc setup  ·  Q quit",
    )
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

fn draw_board(frame: &mut Frame<'_>, app: &App, area: Rect, show_cursor: bool) {
    let block = Block::default()
        .title(format!(
            "Board · seed: {} · finite edges are dead",
            app.seed_name
        ))
        .borders(Borders::ALL);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.width == 0 || inner.height == 0 {
        return;
    }

    let visible_width = (inner.width as usize).min(app.board.width());
    let visible_height = (inner.height as usize).min(app.board.height());
    let max_x = app.board.width().saturating_sub(visible_width);
    let max_y = app.board.height().saturating_sub(visible_height);
    let focus = if show_cursor {
        app.cursor
    } else {
        (app.board.width() / 2, app.board.height() / 2)
    };
    let offset_x = focus.0.saturating_sub(visible_width / 2).min(max_x);
    let offset_y = focus.1.saturating_sub(visible_height / 2).min(max_y);

    let rows = (0..visible_height).map(|screen_y| {
        let y = offset_y + screen_y;
        let cells = (0..visible_width).map(|screen_x| {
            let x = offset_x + screen_x;
            let selected = show_cursor && app.cursor == (x, y);
            if app.board.get(x, y) {
                TableCell::from("●").style(Style::default().fg(Color::Green).add_modifier(
                    if selected {
                        Modifier::REVERSED
                    } else {
                        Modifier::BOLD
                    },
                ))
            } else if selected {
                TableCell::from("+").style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                TableCell::from(" ")
            }
        });
        Row::new(cells)
    });
    let widths = vec![Constraint::Length(1); visible_width];
    let table = Table::new(rows, widths).column_spacing(0);
    frame.render_widget(table, inner);
}

#[cfg(test)]
mod tests {
    use super::{handle_key, App, Screen, StartSource};
    use crossterm::event::KeyCode;

    #[test]
    fn editing_a_running_pattern_starts_from_the_edited_workspace() {
        let mut app = App::new();
        app.screen = Screen::Simulation;
        app.source = StartSource::Pattern;

        assert!(!handle_key(&mut app, KeyCode::Char('e')));
        assert_eq!(app.screen, Screen::Workspace);
        assert_eq!(app.source, StartSource::Workspace);

        let (x, y) = app.cursor;
        assert!(!handle_key(&mut app, KeyCode::Char('x')));
        assert!(!handle_key(&mut app, KeyCode::Enter));

        assert_eq!(app.screen, Screen::Simulation);
        assert_eq!(app.generation, 0);
        assert!(app.running);
        assert_eq!(app.board.live_count(), 1);
        assert!(app.board.get(x, y));
        assert_eq!(app.board, app.seed);
    }

    #[test]
    fn single_step_advances_once_and_pauses() {
        let mut app = App::new();
        app.screen = Screen::Simulation;
        app.running = true;

        assert!(!handle_key(&mut app, KeyCode::Char('s')));

        assert_eq!(app.generation, 1);
        assert!(!app.running);
    }

    #[test]
    fn q_quits_from_the_simulation() {
        let mut app = App::new();
        app.screen = Screen::Simulation;

        assert!(handle_key(&mut app, KeyCode::Char('q')));
    }
}
