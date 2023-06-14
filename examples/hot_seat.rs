use std::{thread, time};

use console::{Key, Term};
use flume::{unbounded, Receiver};
use gametetris_rs::{
    Action, AnsiTermStyle, GameFieldPair, PlayerSide, StepResult, TermRender, TetrisPair,
    TetrisPairState,
};

fn start_tetris_thread(
    player_actions: Receiver<Action>,
    opponent_actions: Receiver<Action>,
) -> Receiver<TetrisPairState> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let mut tetris_pair = TetrisPair::new(10, 20);

        // Setup ganme speed
        let step_delay = time::Duration::from_millis(10);
        tetris_pair.set_fall_speed(1, 30);
        tetris_pair.set_drop_speed(1, 1);
        tetris_pair.set_line_remove_speed(3, 5);

        loop {
            let start = time::Instant::now();
            while let Ok(action) = player_actions.try_recv() {
                tetris_pair.add_player_action(PlayerSide::Player, action);
            }
            while let Ok(action) = opponent_actions.try_recv() {
                tetris_pair.add_player_action(PlayerSide::Opponent, action);
            }

            if tetris_pair.step() != (StepResult::None, StepResult::None) {
                tx.send(tetris_pair.get_state()).unwrap();
            }

            let elapsed = start.elapsed();
            if elapsed < step_delay {
                thread::sleep(step_delay - elapsed);
            }
        }
    });
    rx
}

fn start_read_key_thread() -> (Receiver<Action>, Receiver<Action>) {
    let term = Term::stdout();
    let (tx_player, rx_player) = unbounded();
    let (tx_opponent, rx_opponent) = unbounded();
    thread::spawn(move || loop {
        let key = term.read_key().unwrap();
        if let Some(action) = key_to_action_player(&key) {
            tx_player.send(action).unwrap();
        }
        if let Some(action) = key_to_action_opponent(&key) {
            tx_opponent.send(action).unwrap();
        }
    });
    (rx_player, rx_opponent)
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

    let (action_rx_player, action_rx_opponent) = start_read_key_thread();
    let state_rx = start_tetris_thread(
        action_rx_player,
        action_rx_opponent,
    );

    term.clear_screen().unwrap();
    while let Ok(state) = state_rx.recv() {
        // Draw tetris field on term
        let field = GameFieldPair::new(state, vec!["PLAYER".to_string()], vec!["OPPONENT".to_string()]);
        let lines = field.render(&AnsiTermStyle);
        term.move_cursor_to(0, 0).unwrap();
        for line in lines {
            term.write_line(&line).unwrap();
        }
    }
}
