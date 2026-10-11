use wasm_bindgen::prelude::*;

use crate::grid::{Board, PATTERNS};

#[wasm_bindgen]
pub struct WebBoard {
    board: Board,
    seed: Board,
    generation: u64,
}

#[wasm_bindgen]
impl WebBoard {
    #[wasm_bindgen(constructor)]
    pub fn new(width: usize, height: usize) -> Self {
        let board = Board::new(width, height);
        Self {
            seed: board.clone(),
            board,
            generation: 0,
        }
    }

    pub fn width(&self) -> usize {
        self.board.width()
    }

    pub fn height(&self) -> usize {
        self.board.height()
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn live_count(&self) -> usize {
        self.board.live_count()
    }

    pub fn clear(&mut self) {
        self.board.clear();
        self.generation = 0;
    }

    pub fn save_seed(&mut self) {
        self.seed = self.board.clone();
        self.generation = 0;
    }

    pub fn reset_to_seed(&mut self) {
        self.board = self.seed.clone();
        self.generation = 0;
    }

    pub fn toggle_cell(&mut self, x: usize, y: usize) {
        self.board.toggle(x, y);
    }

    pub fn set_cell(&mut self, x: usize, y: usize, alive: bool) {
        self.board.set(x, y, alive);
    }

    pub fn cell(&self, x: usize, y: usize) -> bool {
        self.board.get(x, y)
    }

    pub fn step(&mut self) {
        self.board.step();
        self.generation = self.generation.saturating_add(1);
    }

    pub fn reset_generation(&mut self) {
        self.generation = 0;
    }

    pub fn load_pattern(&mut self, pattern_index: usize) -> bool {
        let Some(pattern) = PATTERNS.get(pattern_index) else {
            return false;
        };
        self.board.load_pattern(pattern);
        self.seed = self.board.clone();
        self.generation = 0;
        true
    }

    pub fn randomize(&mut self, seed: u32) {
        let mut state = u64::from(seed).max(1);
        for y in 0..self.board.height() {
            for x in 0..self.board.width() {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                self.board.set(x, y, state & 1 == 1);
            }
        }
        self.seed = self.board.clone();
        self.generation = 0;
    }

    pub fn cells(&self) -> Vec<u8> {
        self.board
            .cells()
            .iter()
            .map(|&alive| u8::from(alive))
            .collect()
    }

    pub fn pattern_count() -> usize {
        PATTERNS.len()
    }

    pub fn pattern_name(index: usize) -> String {
        PATTERNS
            .get(index)
            .map(|pattern| pattern.name.to_string())
            .unwrap_or_default()
    }

    pub fn pattern_description(index: usize) -> String {
        PATTERNS
            .get(index)
            .map(|pattern| pattern.description.to_string())
            .unwrap_or_default()
    }
}
