use std::{thread, time};

use console::{Key, Term};
use flume::{unbounded, Receiver};
use gametetris_rs::{
    Action, AnsiTermStyle, GameFieldPair, PlayerSide, StepResult, TermRender, TetrisPair,
    TetrisPairState,
};
use human_hash::humanize;
use zenoh::{
    prelude::{sync::SyncResolve, Config, KeyExpr},
    queryable::{Query, Queryable},
    sample::Sample,
    subscriber::Subscriber,
    Session,
};

fn start_tetris_thread(
    player_name: String,
    opponent_name: String,
    player_actions: Receiver<Action>,
    opponent_actions: Receiver<Action>,
) -> Receiver<TetrisPairState> {
    let (tx, rx) = unbounded();
    thread::spawn(move || {
        let mut tetris_pair = TetrisPair::new(player_name, opponent_name, 10, 20);

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

fn start_read_key_thread() -> Receiver<Action> {
    let term = Term::stdout();
    let (tx_player, rx_player) = unbounded();
    thread::spawn(move || loop {
        let key = term.read_key().unwrap();
        if let Some(action) = key_to_action_player(&key) {
            tx_player.send(action).unwrap();
        }
    });
    rx_player
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

fn start_opponent_action_subscriber<'a>(
    session: &'a Session,
    server_keyexpr: &KeyExpr,
) -> (Subscriber<'a, ()>, Receiver<Action>) {
    let (tx_opponent, rx_opponent) = unbounded();
    let callback = move |sample: Sample| {
        let value = sample.value.to_string();
        let action: Action = serde_json::from_str(&value).unwrap();
        tx_opponent.send(action).unwrap();
    };
    let subscriber = session
        .declare_subscriber(server_keyexpr)
        .callback_mut(callback)
        .res()
        .unwrap();
    (subscriber, rx_opponent)
}

fn start_game_server_queryable<'a>(
    session: &'a Session,
    server_keyexpr: &KeyExpr,
) -> Queryable<'a, ()> {
    let callback = {
        let server_keyexpr = server_keyexpr.clone().into_owned();
        move |query: Query| {
            let sample = Sample::new(server_keyexpr.clone(), "".to_string());
            query.reply(Ok(sample)).res_sync().unwrap();
        }
    };
    let queryable = session
        .declare_queryable(server_keyexpr)
        .callback(callback)
        .res()
        .unwrap();
    queryable
}

fn main() {
    let term = Term::stdout();

    // Generate two random player names
    let server_name = humanize(&uuid::Uuid::new_v4(), 2);

    let config = Config::default();
    let session = zenoh::open(config).res().unwrap();
    let server_keyexpr = format!("tetris/{}", server_name);
    let server_keyexpr = KeyExpr::new(&server_keyexpr).unwrap();
    let action_keyexpr = format!("tetris/{}/action", server_name);
    let action_keyexpr = KeyExpr::new(&action_keyexpr).unwrap();
    let gamestate_keyexpr = format!("tetris/{}/gamestate", server_name);
    let gamestate_keyexpr = KeyExpr::new(&gamestate_keyexpr).unwrap();

    let _queyable = start_game_server_queryable(&session, &server_keyexpr);

    let (_subscriber, action_rx_opponent) =
        start_opponent_action_subscriber(&session, &action_keyexpr);

    let publisher = session.declare_publisher(&gamestate_keyexpr).res().unwrap();

    let action_rx_player = start_read_key_thread();
    let state_rx = start_tetris_thread(
        server_name.clone(),
        "REMOTE".to_string(),
        action_rx_player,
        action_rx_opponent,
    );

    term.clear_screen().unwrap();
    while let Ok(state) = state_rx.recv() {
        // Publish state
        let value = serde_json::to_string(&state).unwrap();
        publisher.put(value).res().unwrap();

        // Draw tetris field on term
        let field: GameFieldPair = state.into();
        let lines = field.render(&AnsiTermStyle);
        term.move_cursor_to(0, 0).unwrap();
        for line in lines {
            term.write_line(&line).unwrap();
        }
    }
}
