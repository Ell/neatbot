use anyhow::{anyhow, Result};
use futures::StreamExt;
use futures::{stream::SelectAll, SinkExt};
use irc_rust::Message;
use std::collections::HashMap;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

use tokio_util::codec::{Framed, LinesCodec};

#[derive(Debug)]
pub struct ConnectionManager {
    connections: HashMap<String, Connection>,
    pub messages: SelectAll<Receiver<Event>>,
}

impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        ConnectionManager {
            connections: HashMap::new(),
            messages: SelectAll::new(),
        }
    }

    pub fn add_connection(&mut self, name: &str, host: &str, port: u16, ssl: bool) {
        let connection = Connection::new(name, host, port, ssl);

        self.connections.insert(name.to_string(), connection);
    }

    pub async fn start(&mut self) {
        for (_, v) in self.connections.iter_mut() {
            let event_stream = v.connect().await.unwrap();

            self.messages.push(event_stream);
        }
    }
}

#[derive(Debug)]
pub enum Event {
    IRC(String, Message),
    Connection(String, ConnectionEvent),
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Error(String),
    Connected,
    Disconnected,
}

#[derive(Debug)]
pub struct Connection {
    name: String,
    host: String,
    ssl: bool,
    port: u16,
    msg_tx: Option<Sender<Message>>,
}

impl Connection {
    pub fn new(name: &str, host: &str, port: u16, ssl: bool) -> Connection {
        Self {
            name: name.to_string(),
            host: host.to_string(),
            msg_tx: None,
            ssl,
            port,
        }
    }

    pub async fn connect(&mut self) -> Result<Receiver<Event>> {
        let (msg_tx, mut msg_rx) = mpsc::channel::<Message>(32);
        let (event_tx, event_rx) = mpsc::channel::<Event>(32);

        self.msg_tx = Some(msg_tx);

        let host: String = self.host.clone() + ":" + &self.port.to_string();
        let stream = TcpStream::connect(host).await.unwrap();

        let (mut sink, mut stream) =
            Framed::new(stream, LinesCodec::new_with_max_length(1024)).split();

        event_tx
            .send(Event::Connection(
                self.name.clone(),
                ConnectionEvent::Connected,
            ))
            .await
            .ok();

        tokio::spawn(async move {
            while let Some(message) = msg_rx.recv().await {
                sink.send(message.to_string()).await.ok();
            }
        });

        let connection_name = self.name.clone();

        tokio::spawn(async move {
            while let Some(result) = stream.next().await {
                match result {
                    Ok(line) => {
                        let irc_message = Message::from(line);
                        event_tx
                            .send(Event::IRC(connection_name.clone(), irc_message))
                            .await
                            .ok();
                    }
                    Err(e) => {
                        event_tx
                            .send(Event::Connection(
                                connection_name.clone(),
                                ConnectionEvent::Error(e.to_string()),
                            ))
                            .await
                            .ok();
                    }
                }
            }

            event_tx
                .send(Event::Connection(
                    connection_name.clone(),
                    ConnectionEvent::Disconnected,
                ))
                .await
                .ok();
        });

        Ok(event_rx)
    }

    pub async fn send_message(&self, message: Message) -> Result<()> {
        if let Some(msg_tx) = &self.msg_tx {
            msg_tx.send(message).await?;
            Ok(())
        } else {
            Err(anyhow!("Invalid Connection"))
        }
    }
}
