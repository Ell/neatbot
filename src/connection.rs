use anyhow::Result;
use futures::{SinkExt, StreamExt};
use irc_rust::Message;
use tokio::sync::broadcast;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_util::codec::{Framed, LinesCodec};

use crate::config::{Config, ServerConfig};

#[derive(Debug, Clone)]
pub enum ConnectionEvent {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum Event {
    Connection(ConnectionEvent),
    Message(Message),
}

#[derive(Debug, Clone)]
pub enum ConnectionCommand {
    Disconnect,
    Reconnect,
}

#[derive(Debug, Clone)]
pub enum Command {
    Message(Message),
    Connection(ConnectionCommand),
    Startup,
}

#[derive(Debug, Clone)]
pub struct TaggedEvent {
    pub name: String,
    pub event: Event,
}

#[derive(Debug, Clone)]
pub struct TaggedCommand {
    pub name: String,
    pub command: Command,
}

impl TaggedCommand {
    pub fn new(server_name: &str, command: Command) -> Self {
        Self {
            name: server_name.to_string(),
            command,
        }
    }

    pub fn new_message(server_name: &str, message: Message) -> Self {
        Self {
            name: server_name.to_string(),
            command: Command::Message(message),
        }
    }

    pub fn new_connection(server_name: &str, connection: ConnectionCommand) -> Self {
        Self {
            name: server_name.to_string(),
            command: Command::Connection(connection),
        }
    }
}

#[derive(Debug, Default)]
pub struct ConnectionManager {
    connections: Vec<Connection>,
}

impl ConnectionManager {
    pub fn new(config: &Config) -> Result<ConnectionManager> {
        let mut manager = ConnectionManager {
            ..Default::default()
        };

        &config.server.iter().map(|c| manager.add_server(c));

        Ok(manager)
    }

    pub fn add_server(&mut self, config: &ServerConfig) -> Result<()> {
        let host = config.host.clone() + ":" + &config.port.to_string();

        let connection = Connection::new(&config.name, &host, config.ssl);

        self.connections.push(connection);

        Ok(())
    }

    pub async fn start(
        &mut self,
    ) -> Result<(
        broadcast::Sender<TaggedCommand>,
        mpsc::Receiver<TaggedEvent>,
    )> {
        let (event_tx, event_rx) = mpsc::channel::<TaggedEvent>(32);
        let (command_tx, _) = broadcast::channel::<TaggedCommand>(32);

        for conn in &mut self.connections {
            conn.connect(event_tx.clone(), command_tx.subscribe())
                .await
                .unwrap();
        }

        Ok((command_tx.clone(), event_rx))
    }
}

#[derive(Debug, Default)]
pub struct Connection {
    name: String,
    host: String,
    ssl: bool,
}

impl Connection {
    pub fn new(name: &str, host: &str, ssl: bool) -> Connection {
        Self {
            name: name.to_string(),
            host: host.to_string(),
            ssl,
            ..Default::default()
        }
    }

    pub async fn connect(
        &mut self,
        event_tx: mpsc::Sender<TaggedEvent>,
        mut command_rx: broadcast::Receiver<TaggedCommand>,
    ) -> Result<()> {
        let tcp_stream = TcpStream::connect(&self.host).await?;

        let codec = LinesCodec::new_with_max_length(1024);
        let (mut sink, mut stream) = Framed::new(tcp_stream, codec).split();

        let name = self.name.clone();

        tokio::spawn(async move {
            event_tx
                .clone()
                .send(TaggedEvent {
                    name: name.clone(),
                    event: Event::Connection(ConnectionEvent::Connected),
                })
                .await
                .ok();

            while let Some(result) = stream.next().await {
                if let Ok(event) = result {
                    let message = Message::from(event);

                    println!("<< {:?}", message.clone());

                    let event = TaggedEvent {
                        name: name.clone(),
                        event: Event::Message(message),
                    };

                    event_tx.clone().send(event).await.ok();
                }
            }

            event_tx
                .clone()
                .send(TaggedEvent {
                    name: name.clone(),
                    event: Event::Connection(ConnectionEvent::Disconnected),
                })
                .await
                .ok();
        });

        let conn_name = self.name.clone();
        tokio::spawn(async move {
            while let Ok(tagged_command) = command_rx.recv().await {
                let name = tagged_command.name;
                let command = tagged_command.command;

                if conn_name == name {
                    match command {
                        Command::Message(message) => {
                            println!(">> {:?}", message);

                            sink.send(message.to_string()).await.ok()
                        }
                        Command::Connection(_) => Some(()),
                        Command::Startup => Some(()),
                    };
                };
            }
        });

        Ok(())
    }
}
