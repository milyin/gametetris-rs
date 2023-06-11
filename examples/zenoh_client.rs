use human_hash::humanize;
use zenoh::{
    prelude::{r#async::AsyncResolve, Config},
    query::Reply,
};

#[async_std::main]
async fn main() {
    let config = Config::default();
    let session = zenoh::open(config).res_async().await.unwrap();
    let player_name = humanize(&uuid::Uuid::new_v4(), 2);
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
    println!("Selected player: {}", opponent_name);
}
