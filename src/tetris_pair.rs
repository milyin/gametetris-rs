use crate::tetris::{Action, StepResult, Tetris, TetrisGameState};
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum PlayerSide {
    A,
    B,
}

#[derive(Serialize)]
pub struct TetrisPairState {
    pub player: TetrisGameState,
    pub opponent: TetrisGameState,
}

#[derive(Default)]
pub struct TetrisPair {
    tetris_a: Tetris,
    tetris_b: Tetris,
    // The step is performed when both players have called step method
    // This is to prevent one player from getting an advantage by calling step more often
    step_a: bool,
    step_b: bool,
    step_divergence: usize,
}

impl TetrisPair {
    pub fn new(width: usize, height: usize) -> TetrisPair {
        TetrisPair {
            tetris_a: Tetris::new(width, height),
            tetris_b: Tetris::new(width, height),
            step_a: false,
            step_b: false,
            step_divergence: 0,
        }
    }

    pub fn set_fall_speed(&mut self, lines: usize, steps: usize) {
        self.tetris_a.set_fall_speed(lines, steps);
        self.tetris_b.set_fall_speed(lines, steps);
    }

    pub fn set_drop_speed(&mut self, lines: usize, steps: usize) {
        self.tetris_a.set_drop_speed(lines, steps);
        self.tetris_b.set_drop_speed(lines, steps);
    }

    pub fn set_line_remove_speed(&mut self, lines: usize, steps: usize) {
        self.tetris_a.set_line_remove_speed(lines, steps);
        self.tetris_b.set_line_remove_speed(lines, steps);
    }

    pub fn step(&mut self) {
        self.step_a = false;
        self.step_b = false;
        let step_result_a = self.tetris_a.step();
        let step_result_b = self.tetris_b.step();
        if step_result_a == StepResult::LineRemoved {
            self.tetris_b.add_action(Action::BottomRefill);
        }
        if step_result_b == StepResult::LineRemoved {
            self.tetris_a.add_action(Action::BottomRefill);
        }
    }

    /// Use this method when players have different control loops
    /// This guarantees that the game will run on frequiency of the slowest player
    pub fn step_player(&mut self, player: PlayerSide) -> usize {
        match player {
            PlayerSide::A => self.step_a = true,
            PlayerSide::B => self.step_b = true,
        }
        if self.step_a && self.step_b {
            self.step();
            self.step_divergence = 0;
        } else {
            self.step_divergence += 1;
        }
        self.step_divergence
    }

    pub fn add_player_action(&mut self, player: PlayerSide, action: Action) {
        match player {
            PlayerSide::A => self.tetris_a.add_action(action),
            PlayerSide::B => self.tetris_b.add_action(action),
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.tetris_a.is_game_over() || self.tetris_b.is_game_over()
    }

    pub fn get_player_game_state(&self, player: PlayerSide) -> TetrisPairState {
        match player {
            PlayerSide::A => TetrisPairState {
                player: self.tetris_a.get_game_state(),
                opponent: self.tetris_b.get_game_state(),
            },
            PlayerSide::B => TetrisPairState {
                player: self.tetris_b.get_game_state(),
                opponent: self.tetris_a.get_game_state(),
            },
        }
    }
}
