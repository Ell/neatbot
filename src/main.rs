mod client;
mod config;
mod connection;
mod irc;
mod plugin;

use std::collections::HashMap;

use anyhow::Result;
use client::Client;
use config::Config;
use connection::ConnectionManager;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<()> {
    let bot_config = Config::from_config_folder().unwrap();

    let mut clients: HashMap<String, Client> = HashMap::new();
    &bot_config.server.iter().for_each(|server_config| {
        let client = Client::new(server_config).unwrap();
        clients.insert(server_config.name.clone(), client);
    });

    let (conn_command_tx, mut conn_event_rx) = ConnectionManager::new(&bot_config)
        .unwrap()
        .start()
        .await
        .unwrap();

    while let Some(event) = conn_event_rx.next().await {
        if let Some(client) = clients.get_mut(&event.name) {
            let command_tx = conn_command_tx.clone();

            client
                .handle_event(event.clone(), command_tx.clone())
                .await
                .ok();
        }
    }

    Ok(())
}
