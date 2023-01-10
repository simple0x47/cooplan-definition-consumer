use async_channel::Sender;
use cooplan_auth::client_data::ClientData;
use cooplan_auth::identity::Identity;
use cooplan_definition_consumer::error::{Error, ErrorKind};
use cooplan_definition_consumer::initialize_api_consumer;
use cooplan_definition_consumer::logic::actions::definition_storage_action::DefinitionStorageAction;
use cooplan_definitions_lib::definition::Definition;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;
use cooplan_state_tracker::tracked_data::TrackedData;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[cfg(test)]

async fn setup() -> Result<Sender<DefinitionStorageAction>, Error> {
    const CONSUMER_CONFIG_FILE_ENV: &str = "CONSUMER_CONFIG_FILE";
    const API_CONSUMER_FILE_ENV: &str = "API_CONSUMER_FILE";
    const IDENTITY_PROVIDER_URL_ENV: &str = "IDENTITY_PROVIDER_URL";
    const IDENTITY_CLIENT_ID_ENV: &str = "IDENTITY_CLIENT_ID";
    const IDENTITY_CLIENT_SECRET_ENV: &str = "IDENTITY_CLIENT_SECRET";
    const IDENTITY_CLIENT_AUDIENCE_ENV: &str = "IDENTITY_CLIENT_AUDIENCE";

    let consumer_config_file = match std::env::var(CONSUMER_CONFIG_FILE_ENV) {
        Ok(consumer_config_file) => consumer_config_file,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to get consumer config file from env: {}", error),
            ))
        }
    };

    let api_consumer_file = match std::env::var(API_CONSUMER_FILE_ENV) {
        Ok(api_consumer_file) => api_consumer_file,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to get api consumer file from env: {}", error),
            ))
        }
    };

    let (state_sender, state_receiver) = tokio::sync::mpsc::channel::<TrackedData>(1024);
    let state_tracker_client = StateTrackerClient::new("test".to_string(), state_sender, 15u64);

    let identity_provider_url = match std::env::var(IDENTITY_PROVIDER_URL_ENV) {
        Ok(identity_provider_url) => identity_provider_url,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!(
                    "failed to read environment variable '{}': {}",
                    IDENTITY_PROVIDER_URL_ENV, error
                ),
            ))
        }
    };

    let identity_client_id = match std::env::var(IDENTITY_CLIENT_ID_ENV) {
        Ok(identity_client_id) => identity_client_id,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!(
                    "failed to read environment variable '{}': {}",
                    IDENTITY_CLIENT_ID_ENV, error
                ),
            ))
        }
    };

    let identity_client_secret = match std::env::var(IDENTITY_CLIENT_SECRET_ENV) {
        Ok(identity_client_secret) => identity_client_secret,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!(
                    "failed to read environment variable '{}': {}",
                    IDENTITY_CLIENT_SECRET_ENV, error
                ),
            ))
        }
    };

    let identity_client_audience = match std::env::var(IDENTITY_CLIENT_AUDIENCE_ENV) {
        Ok(identity_client_audience) => identity_client_audience,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!(
                    "failed to read environment variable '{}': {}",
                    IDENTITY_CLIENT_AUDIENCE_ENV, error
                ),
            ))
        }
    };

    let client_data = ClientData::new(
        identity_client_id,
        identity_client_secret,
        identity_client_audience,
    );

    let identity = match Identity::try_new(identity_provider_url, client_data).await {
        Ok(identity) => identity,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to create identity: {}", error),
            ))
        }
    };

    let api_consumer = match initialize_api_consumer(
        &consumer_config_file,
        &api_consumer_file,
        state_tracker_client,
        Arc::new(identity),
    )
    .await
    {
        Ok(api_consumer) => api_consumer,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::InternalFailure,
                format!("failed to initialize api consumer: {}", error),
            ))
        }
    };

    Ok(api_consumer)
}

#[tokio::test]
pub async fn get_latest_correctly() {
    const TIMEOUT_AFTER_SECONDS: u64 = 5u64;

    let api_consumer = setup().await.unwrap();
    let (sender, receiver) = tokio::sync::oneshot::channel::<
        Result<Definition, cooplan_amqp_api_consumer::error::Error>,
    >();

    let action = DefinitionStorageAction::FindLatest { replier: sender };

    api_consumer.send(action).await.unwrap();

    let definition = match timeout(Duration::from_secs(TIMEOUT_AFTER_SECONDS), receiver)
        .await
        .unwrap()
        .unwrap()
    {
        Ok(definition) => definition,
        Err(error) => panic!("failed to get definition: {}", error),
    };

    assert!(!definition.version().is_empty());
    assert!(!definition.categories().is_empty());
}

#[tokio::test]
pub async fn get_by_version_correctly() {
    const TIMEOUT_AFTER_SECONDS: u64 = 5u64;
    const VERSION: &str = "6bc4697553f0511780480ddae602a636802d3cdc";

    let api_consumer = setup().await.unwrap();
    let (sender, receiver) = tokio::sync::oneshot::channel::<
        Result<Definition, cooplan_amqp_api_consumer::error::Error>,
    >();

    let action = DefinitionStorageAction::FindByVersion {
        version: VERSION.to_string(),
        replier: sender,
    };

    api_consumer.send(action).await.unwrap();

    let definition = match timeout(Duration::from_secs(TIMEOUT_AFTER_SECONDS), receiver)
        .await
        .unwrap()
        .unwrap()
    {
        Ok(definition) => definition,
        Err(error) => panic!("failed to get definition: {}", error),
    };

    assert!(!definition.version().is_empty());
    assert_eq!(definition.version(), VERSION);
    assert!(!definition.categories().is_empty());
}
