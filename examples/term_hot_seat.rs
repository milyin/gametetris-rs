use std::{
    sync::mpsc::{channel, Receiver},
    thread, time,
};

use console::{Key, Term};
use gametetris_rs::{
    Action, AnsiTermStyle, GameFieldPair, PlayerSide, StepResult, TermRender, TetrisPair,
};

// function which runs thread for reading key input and returns chamnel with key input
fn read_key_input() -> Receiver<Key> {
    let term = Term::stdout();
    let (tx, rx) = channel();
    thread::spawn(move || loop {
        let key = term.read_key().unwrap();
        tx.send(key).unwrap();
    });
    rx
}

fn key_to_action_player(key: &Key) -> Option<Action> {
    match key {
        Key::ArrowLeft => Some(Action::MoveLeft),
        Key::ArrowRight => Some(Action::MoveRight),
        Key::ArrowDown => Some(Action::MoveDown),
        Key::ArrowUp => Some(Action::RotateLeft),
        Key::Char(' ') => Some(Action::Drop),
        _ => None,
    }
}

fn key_to_action_opponent(key: &Key) -> Option<Action> {
    match key {
        Key::Char('a') => Some(Action::MoveLeft),
        Key::Char('d') => Some(Action::MoveRight),
        Key::Char('s') => Some(Action::MoveDown),
        Key::Char('w') => Some(Action::RotateLeft),
        Key::Char('q') => Some(Action::Drop),
        _ => None,
    }
}

fn main() {
    let term = Term::stdout();

    let key_rx = read_key_input();
    let mut tetris_pair = TetrisPair::default();

    term.clear_screen().unwrap();
    term.hide_cursor().unwrap();
    let style = AnsiTermStyle;

    // Setup ganme speed
    let step_delay = time::Duration::from_millis(10);
    tetris_pair.set_fall_speed(1, 30);
    tetris_pair.set_drop_speed(1, 1);
    tetris_pair.set_line_remove_speed(3, 5);

    loop {
        let start = time::Instant::now();
        // Get key input from term without waiting
        while let Ok(key) = key_rx.try_recv() {
            if let Some(action) = key_to_action_player(&key) {
                tetris_pair.add_player_action(PlayerSide::Player, action);
            }
            if let Some(action) = key_to_action_opponent(&key) {
                tetris_pair.add_player_action(PlayerSide::Opponent, action);
            }
        }
        if tetris_pair.step() != (StepResult::None, StepResult::None) {
            // Draw tetris field on term
            let state = tetris_pair.get_state();
            let field: GameFieldPair = state.into();
            let lines = field.render(&style);
            term.move_cursor_to(0, 0).unwrap();
            for line in lines {
                term.write_line(&line).unwrap();
            }
        }

        let elapsed = start.elapsed();
        if elapsed < step_delay {
            thread::sleep(step_delay - elapsed);
        }
    }
}
