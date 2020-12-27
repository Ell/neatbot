use crate::{
    config::ServerConfig,
    connection::{ConnectionEvent, Event, TaggedCommand, TaggedEvent},
    irc::Channel,
};
use anyhow::Result;
use irc_rust::Message;
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum ClientStatus {
    Connected,
    Disconnected,
}

impl Default for ClientStatus {
    fn default() -> Self {
        Self::Disconnected
    }
}

#[derive(Clone)]
pub struct Client {
    config: ServerConfig,
    nickname: Option<String>,
    channels: Vec<Channel>,
    status: ClientStatus,
}

impl Client {
    pub fn new(config: &ServerConfig) -> Result<Self> {
        let client = Self {
            config: config.clone(),
            nickname: None,
            channels: Vec::new(),
            status: ClientStatus::Disconnected,
        };

        Ok(client)
    }

    pub async fn handle_event(
        &mut self,
        tagged_event: TaggedEvent,
        command_tx: broadcast::Sender<TaggedCommand>,
    ) -> Result<()> {
        match tagged_event.event {
            Event::Connection(event) => {
                self.handle_connection_event(event, command_tx.clone())
                    .await
            }
            Event::Message(message) => self.handle_message_event(message, command_tx.clone()).await,
        }
    }

    async fn handle_message_event(
        &self,
        message: Message,
        command_tx: broadcast::Sender<TaggedCommand>,
    ) -> Result<()> {
        match message.command() {
            "PING" => self.handle_ping(message, command_tx.clone()).await?,
            _ => {}
        }

        Ok(())
    }

    async fn handle_ping(
        &self,
        message: Message,
        command_tx: broadcast::Sender<TaggedCommand>,
    ) -> Result<()> {
        if let Some(params) = message.params() {
            if let Some(trailing) = params.trailing() {
                let ping_command = TaggedCommand::new_message(
                    &self.config.name,
                    Message::from(format!("PONG :{}", trailing)),
                );
                command_tx.send(ping_command)?;
            }
        }

        Ok(())
    }

    async fn handle_connection_event(
        &mut self,
        event: ConnectionEvent,
        command_tx: broadcast::Sender<TaggedCommand>,
    ) -> Result<()> {
        match event {
            ConnectionEvent::Connected => {
                self.status = ClientStatus::Connected;
                self.authorize(command_tx.clone()).await?;
            }
            ConnectionEvent::Disconnected => {
                self.status = ClientStatus::Disconnected;
            }
        };

        Ok(())
    }

    async fn authorize(&mut self, command_tx: broadcast::Sender<TaggedCommand>) -> Result<()> {
        if let Some(nicknames) = &self.config.nicknames {
            if let Some(nickname) = nicknames.first() {
                let nick_command = TaggedCommand::new_message(
                    &self.config.name,
                    Message::from(format!("NICK {}", nickname)),
                );
                command_tx.send(nick_command)?;

                let user_command = TaggedCommand::new_message(
                    &self.config.name,
                    Message::from("USER neatbot 0 * neatbot"),
                );
                command_tx.send(user_command)?;

                self.nickname = Some(nickname.clone());
            }
        };

        Ok(())
    }
}
