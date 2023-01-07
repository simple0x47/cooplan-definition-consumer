use cooplan_lapin_wrapper::config::amqp_connect_config::AmqpConnectConfig;
use serde::Deserialize;

use crate::error::{Error, ErrorKind};

#[derive(Deserialize, Clone)]
pub struct ConsumerConfig {
    amqp_connect_config: AmqpConnectConfig,
    concurrent_request_dispatch_instances: u16,
    token_expiration_margin_in_seconds: u64,
    request_boundary: usize,
}

impl ConsumerConfig {
    pub fn owned_amqp_connect_config(self) -> AmqpConnectConfig {
        self.amqp_connect_config
    }

    pub fn concurrent_request_dispatch_instances(&self) -> u16 {
        self.concurrent_request_dispatch_instances
    }

    pub fn token_expiration_margin_in_seconds(&self) -> u64 {
        self.token_expiration_margin_in_seconds
    }

    pub fn request_boundary(&self) -> usize {
        self.request_boundary
    }
}

pub async fn try_generate(consumer_config_file_path: &str) -> Result<ConsumerConfig, Error> {
    let config = match tokio::fs::read_to_string(consumer_config_file_path).await {
        Ok(serialized_config) => {
            match serde_json::from_str::<ConsumerConfig>(serialized_config.as_str()) {
                Ok(config) => config,
                Err(error) => {
                    return Err(Error::new(
                        ErrorKind::AutoConfigFailure,
                        format!("failed to deserialize config file's content: {}", error),
                    ))
                }
            }
        }
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to read config file: {}", error),
            ))
        }
    };

    Ok(config)
}
