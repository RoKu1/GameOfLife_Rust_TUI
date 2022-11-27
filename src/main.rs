use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use tui::{style::{Color, Style}, widgets::{ Paragraph, BorderType}, layout::Alignment};

mod grid;
use grid::{grid as Grid};

use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
        widgets::{
        canvas::{Canvas},
        Block, Borders,
    },
    Frame, Terminal,
};


fn main() -> Result<(), Box<dyn Error>> {
    //setup logger
    // TermLogger::init(LevelFilter::Trace, Config::default(), TerminalMode::Stderr, ColorChoice::Auto).unwrap();
    // debug!("Starting Game Of Life");
    // setup terminal
    enable_raw_mode().expect("can run in raw mode");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // // create app and run it
    let tick_rate = Duration::from_millis(200);
    let grid = Grid::new();
    let res = run_app(&mut terminal, grid, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }


    

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: Grid,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut auto = false;
    let _ = terminal.clear();
    loop {
        terminal.draw(|f| ui(f, &app, true))?;
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        // info!("Quitting GOL");
                        return Ok(());}
                    KeyCode::Char('s') => {
                        // info!("Stepping GOL");
                        app.on_tick(true);
                    }
                    KeyCode::Char('e') => {
                        // info!("Ending GOL");
                        auto = false;

                    } 
                    KeyCode::Char('r') => {
                        auto = false;
                        app.random_gen();
                    } 
                    KeyCode::Char('a') => {
                        auto = true;
                    }
                    _ => {}
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            if auto{
                app.on_tick(true);
            }
            last_tick = Instant::now();
        }

        
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &Grid, active: bool) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(6), Constraint::Percentage(4)].as_ref())
        .split(f.size());    

    ////Creating GOL canvas no
    let canvas = Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("Game Of Life"))
        .paint(|ctx| {
            if active{
            ctx.draw(app);
            }
            // ctx.draw(Cell)
        })
        .x_bounds([0.0, 100.0])
        .y_bounds([0.0, 100.0]);

       
        f.render_widget(canvas, chunks[0]);
        
    

    let instructions = Paragraph::new("Press: r: random, a: auto, e: end, s: 1step, q:quit")
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Instructions")
            .border_type(BorderType::Plain),
    );
    f.render_widget(instructions, chunks[1]);

    let instructions = Paragraph::new("RoKu: Rohit Kulkarni")
    .style(Style::default().fg(Color::Green))
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Author")
            .border_type(BorderType::Plain),
    );
    f.render_widget(instructions, chunks[2]);




}