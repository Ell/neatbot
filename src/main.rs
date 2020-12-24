mod config;
mod connection;
mod irc;

use anyhow::Result;
use config::Config;
use connection::ConnectionManager;

#[tokio::main]
async fn main() -> Result<()> {
    let bot_config = Config::from_config_folder().unwrap();

    let mut connection_manager = ConnectionManager::new();

    bot_config.server.iter().for_each(|server| {
        let host = server.host.clone() + ":" + &server.port.to_string();

        &connection_manager.add_server(&server.name, &host, server.ssl);
    });

    let (command_tx, mut event_rx) = connection_manager.start().await.unwrap();

    while let Some(event) = event_rx.recv().await {
        println!("{:?}", event);
    }

    Ok(())
}
