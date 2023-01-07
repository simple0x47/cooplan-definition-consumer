use crate::config::consumer_config::ConsumerConfig;
use crate::error::Error;
use crate::logic::actions::definition_storage_action::DefinitionStorageAction;
use crate::storage::definition_request_dispatch::DefinitionRequestDispatch;
use async_channel::Receiver;
use cooplan_auth::identity::Identity;
use cooplan_lapin_wrapper::config::api_consumer::ApiConsumer;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;
use std::sync::Arc;

const DEFINITION_REQUEST_BASE_ID: &str = "definition_request_dispatch";

pub async fn initialize(
    request_receiver: Receiver<DefinitionStorageAction>,
    config: ConsumerConfig,
    api_consumer: ApiConsumer,
    identity: Arc<Identity>,
    state_tracker_client: StateTrackerClient,
) -> Result<(), Error> {
    let api_consumer = Arc::new(api_consumer);

    for i in 0..config.concurrent_request_dispatch_instances() {
        let id = format!("{}-{}", DEFINITION_REQUEST_BASE_ID, i);
        let definition_request_dispatch = DefinitionRequestDispatch::try_new(
            id,
            request_receiver.clone(),
            config.clone(),
            api_consumer.clone(),
            identity.clone(),
            state_tracker_client.clone(),
        )
        .await?;

        tokio::spawn(definition_request_dispatch.run());
    }

    Ok(())
}
