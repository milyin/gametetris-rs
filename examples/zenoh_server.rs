use console::Key;
use human_hash::humanize;
use zenoh::{
    prelude::{keyexpr, r#async::AsyncResolve, sync::SyncResolve, Config, KeyExpr},
    queryable::Query,
    sample::Sample,
};

#[async_std::main]
async fn main() {
    let config = Config::default();
    let session = zenoh::open(config).res_async().await.unwrap();
    let player_name = humanize(&uuid::Uuid::new_v4(), 2);
    println!("Player name: {}", player_name);

    let keyexpr = format!("tetris/{}", player_name);
    let keyexpr = KeyExpr::new(keyexpr).unwrap();
    let callback = {
        let keyexpr = keyexpr.clone();
        move |query: Query| {
            println!("Received query: {}", query.key_expr());
            let sample = Sample::new(keyexpr.clone(), player_name.clone());
            query.reply(Ok(sample)).res_sync().unwrap();
        }
    };

    let _queyable = session
        .declare_queryable(&keyexpr)
        .callback(callback)
        .res_async()
        .await
        .unwrap();

    let subscriber = session
        .declare_subscriber(&keyexpr)
        .res_async()
        .await
        .unwrap();

    let opponent_keyexpr = loop {
        if let Ok(sample) = subscriber.recv_async().await {
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

    println!("Selected opponent {}", opponent_keyexpr);
}
