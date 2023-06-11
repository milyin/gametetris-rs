use serde::Serialize;

use crate::Field;

#[derive(Serialize)]
pub struct TetrisState {
    pub well: Field,
    pub preview: Field,
    pub name: String,
    pub game_over: bool,
}

#[derive(Serialize)]
pub struct TetrisPairState {
    pub player: TetrisState,
    pub opponent: TetrisState,
}
