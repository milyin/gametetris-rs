use std::{
    sync::mpsc::{channel, Receiver},
    thread, time,
};

use console::{Key, Term};
use gametetris_rs::{
    Action, AnsiTermStyle, GameFieldPair, PlayerSide, StepResult, TermRender, TetrisPair,
};
use human_hash::humanize;
use zenoh::{
    prelude::{keyexpr, sync::SyncResolve, Config, KeyExpr},
    queryable::Query,
    sample::Sample,
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

fn main() {
    let config = Config::default();
    let session = zenoh::open(config).res_sync().unwrap();
    let player_name = humanize(&uuid::Uuid::new_v4(), 2);
    println!("Player name: {}", player_name);

    let keyexpr = format!("tetris/{}", player_name);
    let keyexpr = KeyExpr::new(keyexpr).unwrap();
    let callback = {
        let keyexpr = keyexpr.clone();
        let player_name = player_name.clone();
        move |query: Query| {
            println!("Received query: {}", query.key_expr());
            let sample = Sample::new(keyexpr.clone(), player_name.clone());
            query.reply(Ok(sample)).res_sync().unwrap();
        }
    };

    let _queyable = session
        .declare_queryable(&keyexpr)
        .callback(callback)
        .res_sync()
        .unwrap();

    let subscriber = session.declare_subscriber(&keyexpr).res_sync().unwrap();

    let opponent_name = loop {
        if let Ok(sample) = subscriber.recv() {
            println!(
                "Player {} requests fight with {}",
                sample.value, sample.key_expr
            );
            if sample.key_expr == keyexpr {
                println!("Request accepted!");
                break sample.value.to_string();
            }
        } else {
            panic!("Listener unexpectedly closed");
        }
    };

    println!("Selected opponent {}", opponent_name);

    let term = Term::stdout();
    let key_rx = read_key_input();
    let mut tetris_pair = TetrisPair::new(player_name, opponent_name, 10, 20);

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
