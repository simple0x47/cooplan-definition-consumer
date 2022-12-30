use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};

use async_recursion::async_recursion;
use lapin::{Channel, Connection};
use tokio::sync::watch::Sender;

use crate::{
    amqp_connect_config::AmqpConnectConfig,
    error::{Error, ErrorKind},
    state::State,
};

struct WrappedConnection {
    pub id: String,
    pub connection: Connection,
}

impl WrappedConnection {
    pub fn new(connection: Connection) -> WrappedConnection {
        WrappedConnection {
            id: uuid::Uuid::new_v4().to_string(),
            connection,
        }
    }
}

impl From<Connection> for WrappedConnection {
    fn from(connection: Connection) -> Self {
        WrappedConnection {
            id: uuid::Uuid::new_v4().to_string(),
            connection,
        }
    }
}

pub struct AmqpWrapper {
    connections: Vec<WrappedConnection>,
    channels_per_connection: HashMap<String, Vec<Weak<Channel>>>,
    state_sender: Sender<State>,
    connect_config: AmqpConnectConfig,
}

impl AmqpWrapper {
    pub fn try_new(
        state_sender: Sender<State>,
        connect_config: AmqpConnectConfig,
    ) -> Result<AmqpWrapper, Error> {
        match state_sender.send(State::Idle) {
            Ok(_) => (),
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::StateUpdateFailure,
                    format!("failed to update state"),
                ))
            }
        }

        Ok(AmqpWrapper {
            connections: Vec::new(),
            channels_per_connection: HashMap::new(),
            state_sender,
            connect_config,
        })
    }

    async fn try_connect(&mut self) -> Result<&WrappedConnection, Error> {
        let connection: WrappedConnection = match Connection::connect_with_config(
            self.connect_config.uri(),
            self.connect_config.cloned_options(),
            self.connect_config.cloned_owned_tls_config(),
        )
        .await
        {
            Ok(connection) => connection.into(),
            Err(error) => {
                let error = Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to connect: {}", error),
                );

                self.state_sender.send(State::Error(error.clone()));

                return Err(error);
            }
        };

        self.connections.push(connection);

        let borrowed_connection = match self.connections.last() {
            Some(connection) => connection,
            None => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to get previously created connection"),
                ))
            }
        };

        Ok(borrowed_connection)
    }

    async fn get_connection(&mut self) -> Result<&WrappedConnection, Error> {
        if self.connections.len() == 0 {
            return self.try_connect().await;
        }

        let last_connection = match self.connections.last() {
            Some(last_connection) => last_connection,
            None => {
                let error = Error::new(
                    ErrorKind::InternalFailure,
                    format!(
                    "failed to get last connection after checking there are available connections"
                ),
                );

                self.state_sender.send(State::Error(error.clone()));

                return Err(error);
            }
        };

        Ok(last_connection)
    }

    #[async_recursion]
    pub async fn try_get_channel(&mut self) -> Result<Arc<Channel>, Error> {
        let wrapped_connection = self.get_connection().await?;
        let connection_id = wrapped_connection.id.clone();

        let channel = match wrapped_connection.connection.create_channel().await {
            Ok(channel) => Arc::new(channel),
            Err(error) => {
                if error == lapin::Error::ChannelsLimitReached {
                    self.try_get_channel_from_new_connection().await?
                } else {
                    let error = Error::new(
                        ErrorKind::InternalFailure,
                        format!("failed to create channel: {}", error),
                    );

                    self.state_sender.send(State::Error(error.clone()));

                    return Err(error);
                }
            }
        };

        let channel_weak = Arc::downgrade(&channel);

        match self.channels_per_connection.get_mut(&connection_id) {
            Some(channels) => channels.push(channel_weak),
            None => {
                self.channels_per_connection
                    .insert(connection_id, vec![channel_weak]);
            }
        }

        match self.state_sender.send(State::Alive) {
            Ok(_) => (),
            Err(_) => {
                return Err(Error::new(
                    ErrorKind::StateUpdateFailure,
                    format!("failed to update state"),
                ))
            }
        }

        Ok(channel)
    }

    async fn try_get_channel_from_new_connection(&mut self) -> Result<Arc<Channel>, Error> {
        self.try_connect().await?;

        let result = self.try_get_channel().await;

        // clean connections that have no channels alive
        self.clean_connections_and_channels().await;

        return result;
    }

    async fn clean_connections_and_channels(&mut self) {
        let mut empty_connection_ids: Vec<&str> = Vec::new();

        for pair in self.channels_per_connection.iter_mut() {
            pair.1
                .retain(|weak_channel| weak_channel.strong_count() > 0);

            if pair.1.is_empty() {
                empty_connection_ids.push(pair.0)
            }
        }

        self.connections.retain(|wrapped_connection| {
            if empty_connection_ids.contains(&wrapped_connection.id.as_str()) {
                false
            } else {
                true
            }
        });
    }
}
