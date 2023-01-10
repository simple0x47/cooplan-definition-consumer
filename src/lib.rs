use crate::config::consumer_config;
use crate::error::{Error, ErrorKind};
use crate::logic::actions::definition_storage_action::DefinitionStorageAction;
use async_channel::Sender;
use cooplan_auth::identity::Identity;
use cooplan_lapin_wrapper::config::api_consumer;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;
use std::sync::Arc;

pub mod config;
pub mod error;
pub mod logic;
pub mod storage;

pub async fn initialize_api_consumer(
    consumer_config_file: &str,
    api_consumer_file: &str,
    state_tracker_client: StateTrackerClient,
    identity: Arc<Identity>,
) -> Result<Sender<DefinitionStorageAction>, Error> {
    let consumer_config = match consumer_config::try_generate(consumer_config_file).await {
        Ok(consumer_config) => consumer_config,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to generate consumer config: {}", error),
            ))
        }
    };

    let api_consumer = match api_consumer::try_get(api_consumer_file).await {
        Ok(api_consumer) => api_consumer,
        Err(error) => {
            return Err(Error::new(
                ErrorKind::AutoConfigFailure,
                format!("failed to generate api consumer config: {}", error),
            ))
        }
    };

    let (sender, receiver) = async_channel::bounded(consumer_config.request_boundary());

    storage::init::initialize(
        receiver,
        consumer_config,
        api_consumer,
        identity,
        state_tracker_client,
    )
    .await?;

    Ok(sender)
}
