use std::{sync::Arc, thread};

use console::{Key, Term};
use gametetris_rs::{Action, AnsiTermStyle, GameFieldPair, TermRender, TetrisPairState};

use zenoh::{
    prelude::{sync::SyncResolve, Config, KeyExpr},
    Session,
};

fn start_read_key_thread(session: Arc<Session>, action_keyexpr: KeyExpr) {
    let term = Term::stdout();
    let action_keyexpr = action_keyexpr.clone().into_owned();
    thread::spawn(move || {
        let publisher = session
            .declare_publisher(&action_keyexpr)
            .res_sync()
            .unwrap();
        loop {
            let key = term.read_key().unwrap();
            if let Some(action) = key_to_action_player(&key) {
                let value = serde_json::to_string(&action).unwrap();
                publisher.put(value).res_sync().unwrap();
            }
        }
    });
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
    let term = Term::stdout();

    let config = Config::default();
    let session = Arc::new(zenoh::open(config).res_sync().unwrap());

    let receiver = session.get("tetris/*").res_sync().unwrap();
    let mut servers = Vec::new();
    while let Ok(reply) = receiver.recv() {
        if let Ok(sample) = reply.sample {
            servers.push(sample.key_expr);
        }
    }
    if servers.len() == 0 {
        println!("No servers found");
        return;
    }
    println!("Select server:");
    for n in 0..servers.len() {
        println!("{}: {}", n, servers[n]);
    }
    let n = loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let n = line.trim().parse::<usize>().unwrap();
        if n < servers.len() {
            break n;
        }
    };
    let server_keyexpr = &servers[n];
    println!("Selected server: {}", server_keyexpr);

    let action_keyexpr = server_keyexpr.join("action").unwrap();
    let gamestate_keyexpr = server_keyexpr.join("gamestate").unwrap();

    start_read_key_thread(session.clone(), action_keyexpr);

    let subscriber = session
        .declare_subscriber(&gamestate_keyexpr)
        .res_sync()
        .unwrap();

    while let Ok(sample) = subscriber.recv() {
        let mut state: TetrisPairState =
            serde_json::from_str(sample.value.to_string().as_str()).unwrap();

        // Draw tetris field on term
        state.swap();
        state.player.name = "PLAYER".to_string();
        let field: GameFieldPair = state.into();
        let lines = field.render(&AnsiTermStyle);
        term.move_cursor_to(0, 0).unwrap();
        for line in lines {
            term.write_line(&line).unwrap();
        }
    }
}
