mod config;
mod connection;

use config::Config;
use connection::ConnectionManager;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let bot_config = match Config::from_config_folder() {
        Ok(config) => config,
        Err(err) => {
            println!("error parsing config: {:?}", err);
            std::process::exit(1);
        }
    };

    let mut manager = ConnectionManager::new();

    bot_config.server.iter().for_each(|server| {
        manager.add_connection(&server.name, &server.host, server.port, server.ssl);
    });

    manager.start().await;

    while let Some(result) = manager.messages.next().await {
        println!("{:?}", result);
    }

    println!("{:?}", bot_config);
}
