use core::panic;
use std::sync::Arc;

use lapin::{tcp::OwnedTLSConfig, Channel, ConnectionProperties};
use tokio::sync::watch;

use crate::{
    amqp_connect_config::AmqpConnectConfig, amqp_wrapper::AmqpWrapper, error::Error, state::State,
};

#[cfg(test)]

const AMQP_URI: &str = "amqp://guest:guest@127.0.0.1:5672";

fn initialize_rabbitmq() {
    use std::process::Command;

    match Command::new("docker")
        .args([
            "run",
            "-it",
            "-d",
            "--rm",
            "--name",
            "rabbitmq",
            "-p",
            "5672:5672",
            "-p",
            "15672:15672",
            "rabbitmq:3.11-management",
        ])
        .output()
    {
        Ok(_) => (),
        Err(error) => panic!("failed to initialize rabbitmq: {}", error),
    }
}

async fn create_channels(amqp_wrapper: &mut AmqpWrapper, amount: usize) -> Vec<Arc<Channel>> {
    let mut channels: Vec<Arc<Channel>> = Vec::new();

    for i in 0..amount {
        let channel = match amqp_wrapper.try_get_channel().await {
            Ok(channel) => channels.push(channel),
            Err(error) => panic!("failed to get channel: {}", error),
        };
    }

    channels
}

#[tokio::test]
async fn test_massive_creation_of_channels() {
    initialize_rabbitmq();

    let properties = ConnectionProperties::default();
    let tls = OwnedTLSConfig {
        cert_chain: None,
        identity: None,
    };

    let config = AmqpConnectConfig::new(AMQP_URI.to_string(), properties, tls);

    let (sender, receiver) = watch::channel(State::Idle);
    let mut amqp_wrapper = match AmqpWrapper::try_new(sender, config) {
        Ok(amqp_wrapper) => amqp_wrapper,
        Err(error) => panic!("failed to initialize amqp wrapper: {}", error),
    };

    let mut channels = create_channels(&mut amqp_wrapper, 1000).await;

    channels.clear();

    channels = create_channels(&mut amqp_wrapper, 1000).await;

    for i in (0..channels.len()).rev() {
        if i % 2 == 0 {}
    }

    create_channels(&mut amqp_wrapper, 1000).await;

    if *receiver.borrow() != State::Alive {
        panic!("amqp wrapper is not alive after creating a massive amount of channels")
    }
}
