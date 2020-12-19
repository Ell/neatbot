use anyhow::Result;
use futures::StreamExt;
use irc_rust::Message;
use std::collections::HashMap;
use tokio::{
    net::TcpStream,
    sync::mpsc::{self, Receiver, Sender},
};

use tokio_util::codec::{Framed, LinesCodec};

#[derive(Debug)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Reconnecting,
    Disconnecting,
    Disconnected,
}

impl Default for ConnectionStatus {
    fn default() -> Self {
        ConnectionStatus::Disconnected
    }
}

#[derive(Debug)]
pub struct ConnectionManager {
    connections: HashMap<String, Connection>,
}

impl ConnectionManager {
    pub fn new() -> ConnectionManager {
        ConnectionManager {
            connections: HashMap::new(),
        }
    }

    pub fn add_connection(&self, name: &str, host: &str, port: &u8) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Connection {
    host: String,
    ssl: bool,
    port: u16,
    status: ConnectionStatus,
    event_tx: Option<Sender<Message>>,
    event_rx: Option<Receiver<Message>>,
    msg_rx: Option<Receiver<Message>>,
    msg_tx: Option<Sender<Message>>,
}

impl Default for Connection {
    fn default() -> Self {
        Self {
            msg_rx: None,
            msg_tx: None,
            event_tx: None,
            event_rx: None,
            host: "localhost".to_string(),
            port: 6667,
            ssl: false,
            status: ConnectionStatus::default(),
        }
    }
}

impl Connection {
    pub fn new(host: &str, port: u16, ssl: bool) -> Result<Connection> {
        let connection = Self {
            host: host.to_string(),
            ssl,
            port,
            ..Default::default()
        };

        Ok(connection)
    }

    pub async fn connect(&mut self) -> Result<()> {
        let (msg_tx, msg_rx) = mpsc::channel::<Message>(32);
        let (event_tx, event_rx) = mpsc::channel::<Message>(32);

        let host: String = self.host.clone() + &self.port.to_string();
        let mut stream = TcpStream::connect(host).await.unwrap();

        let mut framed_stream = Framed::new(stream, LinesCodec::new_with_max_length(1024));

        let (mut framed_writer, mut framed_reader) = framed_stream.split();

        let writer_task = tokio::spawn(async move {
            while let Some(message) = msg_rx.recv().await {
                println!("message {:?}", message);
                framed_writer.send(message.to_string().as_bytes());
            }
        });

        while let Some(result) = framed_reader.next().await {
            match result {
                Ok(line) => {
                    let irc_message = Message::from(line.clone());
                    &self
                        .event_tx
                        .as_ref()
                        .unwrap()
                        .clone()
                        .send(irc_message)
                        .await?;
                }
                Err(_) => (),
            }
        }

        Ok(())
    }

    pub async fn disconnect(&self) -> Result<()> {
        Ok(())
    }

    pub async fn send_message(&self, message: Message) {}
}
