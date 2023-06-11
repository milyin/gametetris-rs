use std::thread;

use async_std::channel::{self, Receiver};
use console::{Key, Term};
use futures::select;
use gametetris_rs::Action;
use human_hash::humanize;
use zenoh::{
    prelude::{r#async::AsyncResolve, Config, KeyExpr},
    query::Reply,
};

// function which runs thread for reading key input and returns chamnel with key input
fn read_key_input() -> Receiver<Key> {
    let term = Term::stdout();
    let (tx, rx) = channel::unbounded();
    thread::spawn(move || loop {
        let key = term.read_key().unwrap();
        tx.send(key)
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

#[async_std::main]
async fn main() {
    let config = Config::default();
    let session = zenoh::open(config).res_async().await.unwrap();
    let player_name = humanize(&uuid::Uuid::new_v4(), 2);
    let player_keyexpr = format!("tetris/{}", player_name);
    let player_keyexpr = KeyExpr::new(player_keyexpr).unwrap();

    println!("Player name: {}", player_name);
    let receiver = session.get("tetris/*").res_async().await.unwrap();
    let mut players = Vec::new();
    while let Ok(reply) = receiver.recv_async().await {
        if let Ok(sample) = reply.sample {
            players.push(sample);
        }
    }
    if players.len() == 0 {
        println!("No players found");
        return;
    }
    println!("Select player:");
    for n in 0..players.len() {
        println!("{}: {}", n, players[n].value);
    }
    let n = loop {
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
        let n = line.trim().parse::<usize>().unwrap();
        if n < players.len() {
            break n;
        }
    };
    let opponent_name: String = players[n].value.to_string();
    let opponent_keyexpr = &players[n].key_expr;
    println!("Selected player: {} at {}", opponent_name, opponent_keyexpr);
    session
        .put(opponent_keyexpr, player_keyexpr.to_string())
        .res_async()
        .await
        .unwrap();

    let key_input = read_key_input();

    loop {
        select!(
            key = key_input.recv_async() => {
                if let Some(action) = key_to_action_player(&key.unwrap()) {
                    tetris_pair.step_player(action);
                }
            },

        )
    }
}
