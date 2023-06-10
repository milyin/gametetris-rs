use std::{
    sync::mpsc::{channel, Receiver},
    thread, time,
};

use console::{Key, Term};
use gametetris_rs::{Action, PlayerSide, TetrisFieldTerm, TetrisPair};

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

fn main() {
    let term = Term::stdout();
    let Some((width,height)) = term.size_checked() else {
        panic!("Cannot get term size");
    };

    let key_rx = read_key_input();
    let mut tetris_pair = TetrisPair::default();
    let mut tetris_term = TetrisFieldTerm::new(0, 0);
    // Setup ganme speed
    let step_delay = time::Duration::from_millis(10);
    tetris_pair.set_fall_speed(1, 30);
    tetris_pair.set_drop_speed(1, 3);
    tetris_pair.set_line_remove_speed(3, 5);
    term.clear_screen().unwrap();
    term.hide_cursor().unwrap();
    loop {
        let start = time::Instant::now();
        // Get key input from term without waiting
        while let Ok(key) = key_rx.try_recv() {
            match key {
                // Move left
                Key::ArrowLeft => tetris_pair.add_player_action(PlayerSide::A, Action::MoveLeft),
                // Move right
                Key::ArrowRight => tetris_pair.add_player_action(PlayerSide::A, Action::MoveRight),
                // Move down
                Key::ArrowDown => tetris_pair.add_player_action(PlayerSide::A, Action::MoveDown),
                // Rotate clockwise
                Key::ArrowUp => tetris_pair.add_player_action(PlayerSide::A, Action::RotateLeft),
                // Drop
                Key::Char(' ') => tetris_pair.add_player_action(PlayerSide::A, Action::Drop),
                // Quit
                Key::Char('q') => break,
                _ => {}
            }
        }
        tetris_pair.step();

        // Draw tetris field on term
        let state = tetris_pair.get_player_game_state(PlayerSide::A);
        tetris_term.update(&term, &state.player.field);

        let elapsed = start.elapsed();
        if elapsed < step_delay {
            thread::sleep(step_delay - elapsed);
        }
    }
}
