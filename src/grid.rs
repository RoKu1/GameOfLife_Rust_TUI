use rand::Rng;

pub const GRID_WIDTH: usize = 100;
pub const GRID_HEIGHT: usize = 100;

#[derive(Clone, Debug)]
pub struct Board {
    width: usize,
    height: usize,
    cells: Vec<bool>,
    next: Vec<bool>,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.width == other.width && self.height == other.height && self.cells == other.cells
    }
}

impl Eq for Board {}

impl Board {
    pub fn new(width: usize, height: usize) -> Self {
        let len = width.saturating_mul(height);
        let mut cells = Vec::with_capacity(len);
        cells.resize(len, false);
        let mut next = Vec::with_capacity(len);
        next.resize(len, false);
        Self {
            width,
            height,
            cells,
            next,
        }
    }

    pub fn game_board() -> Self {
        Self::new(GRID_WIDTH, GRID_HEIGHT)
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> bool {
        self.index(x, y).is_some_and(|index| self.cells[index])
    }

    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        if let Some(index) = self.index(x, y) {
            self.cells[index] = alive;
        }
    }

    pub fn toggle(&mut self, x: usize, y: usize) {
        if let Some(index) = self.index(x, y) {
            self.cells[index] = !self.cells[index];
        }
    }

    pub fn clear(&mut self) {
        self.cells.fill(false);
    }

    pub fn randomize(&mut self) {
        self.randomize_with(&mut rand::rng());
    }

    pub fn randomize_with<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        for cell in &mut self.cells {
            *cell = rng.random();
        }
    }

    pub fn live_count(&self) -> usize {
        self.cells.iter().filter(|&&alive| alive).count()
    }

    pub fn step(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let index = y * self.width + x;
                let neighbors = self.live_neighbors(x, y);
                self.next[index] = if self.cells[index] {
                    (2..=3).contains(&neighbors)
                } else {
                    neighbors == 3
                };
            }
        }
        std::mem::swap(&mut self.cells, &mut self.next);
    }

    pub fn load_pattern(&mut self, pattern: &Pattern) {
        self.clear();
        let origin_x = self.width.saturating_sub(pattern.width) / 2;
        let origin_y = self.height.saturating_sub(pattern.height) / 2;
        for &(x, y) in pattern.live_cells {
            self.set(origin_x + x, origin_y + y, true);
        }
        if let Some(rle) = pattern.rle {
            let (mut x, mut y, mut run) = (0usize, 0usize, 0usize);
            for byte in rle.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
                match byte {
                    b'0'..=b'9' => {
                        run = run
                            .saturating_mul(10)
                            .saturating_add((byte - b'0') as usize);
                    }
                    b'b' | b'o' => {
                        let count = if run == 0 { 1 } else { run };
                        if byte == b'o' {
                            for offset in 0..count {
                                self.set(origin_x + x + offset, origin_y + y, true);
                            }
                        }
                        x = x.saturating_add(count);
                        run = 0;
                    }
                    b'$' => {
                        y = y.saturating_add(if run == 0 { 1 } else { run });
                        x = 0;
                        run = 0;
                    }
                    b'!' => break,
                    _ => {}
                }
            }
        }
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        (x < self.width && y < self.height).then_some(y * self.width + x)
    }

    fn live_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut count = 0;
        for dy in -1isize..=1 {
            for dx in -1isize..=1 {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let nx = x as isize + dx;
                let ny = y as isize + dy;
                if nx >= 0
                    && ny >= 0
                    && (nx as usize) < self.width
                    && (ny as usize) < self.height
                    && self.cells[ny as usize * self.width + nx as usize]
                {
                    count += 1;
                }
            }
        }
        count
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Pattern {
    pub name: &'static str,
    pub description: &'static str,
    pub width: usize,
    pub height: usize,
    pub live_cells: &'static [(usize, usize)],
    pub rle: Option<&'static str>,
}

const BLOCK: &[(usize, usize)] = &[(0, 0), (1, 0), (0, 1), (1, 1)];
const BEEHIVE: &[(usize, usize)] = &[(1, 0), (2, 0), (0, 1), (3, 1), (1, 2), (2, 2)];
const BLINKER: &[(usize, usize)] = &[(0, 1), (1, 1), (2, 1)];
const TOAD: &[(usize, usize)] = &[(1, 0), (2, 0), (3, 0), (0, 1), (1, 1), (2, 1)];
const BEACON: &[(usize, usize)] = &[(0, 0), (1, 0), (0, 1), (3, 2), (2, 3), (3, 3)];
const PULSAR: &[(usize, usize)] = &[
    (2, 0),
    (3, 0),
    (4, 0),
    (8, 0),
    (9, 0),
    (10, 0),
    (0, 2),
    (5, 2),
    (7, 2),
    (12, 2),
    (0, 3),
    (5, 3),
    (7, 3),
    (12, 3),
    (0, 4),
    (5, 4),
    (7, 4),
    (12, 4),
    (2, 5),
    (3, 5),
    (4, 5),
    (8, 5),
    (9, 5),
    (10, 5),
    (2, 7),
    (3, 7),
    (4, 7),
    (8, 7),
    (9, 7),
    (10, 7),
    (0, 8),
    (5, 8),
    (7, 8),
    (12, 8),
    (0, 9),
    (5, 9),
    (7, 9),
    (12, 9),
    (0, 10),
    (5, 10),
    (7, 10),
    (12, 10),
    (2, 12),
    (3, 12),
    (4, 12),
    (8, 12),
    (9, 12),
    (10, 12),
];
const GLIDER: &[(usize, usize)] = &[(1, 0), (2, 1), (0, 2), (1, 2), (2, 2)];
const LWSS: &[(usize, usize)] = &[
    (1, 0),
    (4, 0),
    (0, 1),
    (0, 2),
    (4, 2),
    (0, 3),
    (1, 3),
    (2, 3),
    (3, 3),
];
const R_PENTOMINO: &[(usize, usize)] = &[(1, 0), (2, 0), (0, 1), (1, 1), (1, 2)];
const ACORN: &[(usize, usize)] = &[(1, 0), (3, 1), (0, 2), (1, 2), (4, 2), (5, 2), (6, 2)];

pub const PATTERNS: &[Pattern] = &[
    Pattern {
        name: "Block",
        description: "A stable 2×2 still life",
        width: 2,
        height: 2,
        live_cells: BLOCK,
        rle: None,
    },
    Pattern {
        name: "Beehive",
        description: "A stable six-cell still life",
        width: 4,
        height: 3,
        live_cells: BEEHIVE,
        rle: None,
    },
    Pattern {
        name: "Blinker",
        description: "A period-2 oscillator",
        width: 3,
        height: 3,
        live_cells: BLINKER,
        rle: None,
    },
    Pattern {
        name: "Toad",
        description: "A period-2 oscillator",
        width: 4,
        height: 2,
        live_cells: TOAD,
        rle: None,
    },
    Pattern {
        name: "Beacon",
        description: "A period-2 oscillator",
        width: 4,
        height: 4,
        live_cells: BEACON,
        rle: None,
    },
    Pattern {
        name: "Pulsar",
        description: "A large period-3 oscillator",
        width: 13,
        height: 13,
        live_cells: PULSAR,
        rle: None,
    },
    Pattern {
        name: "Glider",
        description: "A small diagonal spaceship",
        width: 3,
        height: 3,
        live_cells: GLIDER,
        rle: None,
    },
    Pattern {
        name: "LWSS",
        description: "A lightweight spaceship",
        width: 5,
        height: 4,
        live_cells: LWSS,
        rle: None,
    },
    Pattern {
        name: "R-pentomino",
        description: "A small, long-lived seed",
        width: 3,
        height: 3,
        live_cells: R_PENTOMINO,
        rle: None,
    },
    Pattern {
        name: "Acorn",
        description: "A seven-cell long-lived seed",
        width: 7,
        height: 3,
        live_cells: ACORN,
        rle: None,
    },
    Pattern {
        name: "Gosper glider gun",
        description: "Period-30 engine that launches a glider every 30 generations",
        width: 36,
        height: 9,
        live_cells: &[],
        rle: Some("24bo$22bobo$12b2o6b2o$11bo b2o4b2o4b2o$2o8b2o6b2o$2o8b2o4bobo2b2o$10b2o6bobo$11bo b2o4b2o$12b2o!"),
    },
    Pattern {
        name: "Pentadecathlon",
        description: "A fiercely flickering period-15 oscillator",
        width: 10,
        height: 3,
        live_cells: &[],
        rle: Some("2bo4bo$2ob4ob2o$2bo4bo!"),
    },
    Pattern {
        name: "Tumbler",
        description: "A chunky period-14 oscillator with two alternating towers",
        width: 9,
        height: 7,
        live_cells: &[],
        rle: Some("bo5bob$obo3bobo$o2bobo2bo$2bo3bo$2b2ob2o!"),
    },
    Pattern {
        name: "Middleweight spaceship",
        description: "An 11-cell ship that sweeps horizontally across the board",
        width: 6,
        height: 5,
        live_cells: &[],
        rle: Some("3bo$bo3bo$o$o4bo$5o!"),
    },
    Pattern {
        name: "Heavyweight spaceship",
        description: "A broad, fast-moving c/2 spaceship",
        width: 9,
        height: 8,
        live_cells: &[],
        rle: Some("bo$2bo$3o$7b2o$6b2o$2b2o4bo$b2o$3bo!"),
    },
    Pattern {
        name: "Diehard",
        description: "A seven-cell seed that erupts, evolves, then vanishes after 130 generations",
        width: 8,
        height: 3,
        live_cells: &[],
        rle: Some("6bo$2o$bo3b3o!"),
    },
    Pattern {
        name: "Queen bee shuttle",
        description: "A period-30 shuttle sends a queen bee back and forth between blocks",
        width: 22,
        height: 7,
        live_cells: &[],
        rle: Some("9bo$7bobo$6bobo11b2o$2o3bo2bo11b2o$2o4bobo$7bobo$9bo!"),
    },
    Pattern {
        name: "Puffer 1",
        description: "A period-128 engine crawls forward while leaving a trail of debris",
        width: 27,
        height: 7,
        live_cells: &[],
        rle: Some("b3o6bo5bo6b3o$o2bo5b3o3b3o5bo2bo$3bo4b2obo3bob2o4bo$3bo19bo$3bo2bo13bo2bo$3bo2b2o11b2o2bo$2bo3b2o11b2o3bo!"),
    },
    Pattern {
        name: "Pufferfish",
        description: "A compact c/2 puffer that lays down a steadily growing trail",
        width: 15,
        height: 12,
        live_cells: &[],
        rle: Some("3bo7bo$2b3o5b3o$b2o2bo3bo2b2o$3b3o3b3o2$4bo5bo$2bo2bo3bo2bo$o5bobo5bo$2o4bobo4b2o$6bobo$3bobo3bobo$4bo5bo!"),
    },
    Pattern {
        name: "Kok's galaxy",
        description: "A period-8 galaxy that completely reshapes itself every cycle",
        width: 13,
        height: 13,
        live_cells: &[],
        rle: Some("2bo2bobob$2obob3ob$bo6bo$2o5bo$bo5b2o$o6bo$b3obob2o$bobo2bo!"),
    },
];

#[cfg(test)]
mod tests {
    use super::{Board, PATTERNS};
    use rand::{rngs::StdRng, SeedableRng};

    #[test]
    fn block_is_stable() {
        let mut board = Board::new(8, 8);
        board.load_pattern(&PATTERNS[0]);
        let before = board.clone();
        board.step();
        assert_eq!(board, before);
    }

    #[test]
    fn blinker_returns_after_two_generations() {
        let mut board = Board::new(9, 9);
        board.load_pattern(&PATTERNS[2]);
        let initial = board.clone();
        board.step();
        board.step();
        assert_eq!(board, initial);
    }

    #[test]
    fn edges_do_not_wrap() {
        let mut board = Board::new(3, 3);
        board.set(2, 0, true);
        board.set(0, 1, true);
        board.set(1, 1, true);
        board.set(2, 1, true);
        board.step();
        assert!(!board.get(0, 0));
    }

    #[test]
    fn generation_uses_a_synchronous_snapshot() {
        let mut board = Board::new(5, 5);
        board.set(1, 2, true);
        board.set(2, 2, true);
        board.set(3, 2, true);
        board.step();
        assert!(board.get(2, 1));
        assert!(board.get(2, 2));
        assert!(board.get(2, 3));
        assert_eq!(board.live_count(), 3);
    }

    #[test]
    fn patterns_are_centered_and_have_expected_live_count() {
        let mut board = Board::new(20, 20);
        board.load_pattern(&PATTERNS[6]);
        assert_eq!(board.live_count(), 5);
        assert!(board.get(9, 8));
    }

    #[test]
    fn acorn_keeps_seed_alive_for_many_generations() {
        let mut board = Board::new(30, 30);
        board.load_pattern(&PATTERNS[9]);
        for _ in 0..20 {
            board.step();
        }
        assert!(board.live_count() > 0);
    }

    #[test]
    fn glider_moves_one_cell_diagonally_after_four_generations() {
        let mut board = Board::new(20, 20);
        board.load_pattern(&PATTERNS[6]);
        for _ in 0..4 {
            board.step();
        }

        let mut expected = Board::new(20, 20);
        for (x, y) in [(10, 9), (11, 10), (9, 11), (10, 11), (11, 11)] {
            expected.set(x, y, true);
        }
        assert_eq!(board, expected);
    }

    #[test]
    fn seeded_randomization_is_reproducible() {
        let mut first = Board::new(20, 20);
        let mut second = Board::new(20, 20);
        let mut first_rng = StdRng::seed_from_u64(42);
        let mut second_rng = StdRng::seed_from_u64(42);

        first.randomize_with(&mut first_rng);
        second.randomize_with(&mut second_rng);

        assert_eq!(first, second);
    }

    #[test]
    fn out_of_bounds_cell_operations_are_ignored() {
        let mut board = Board::new(3, 3);

        board.set(3, 1, true);
        board.toggle(1, 3);

        assert!(!board.get(3, 1));
        assert!(!board.get(1, 3));
        assert_eq!(board.live_count(), 0);
    }
}
