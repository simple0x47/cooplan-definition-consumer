use crate::config::consumer_config::ConsumerConfig;
use crate::error::{Error, ErrorKind};
use crate::logic::actions::definition_storage_action::DefinitionStorageAction;
use crate::storage::elements;
use async_channel::Receiver;
use cooplan_auth::identity::Identity;
use cooplan_auth::token::Token;
use cooplan_lapin_wrapper::amqp_wrapper::AmqpWrapper;
use cooplan_lapin_wrapper::config::api_consumer::ApiConsumer;
use cooplan_state_tracker::state::State;
use cooplan_state_tracker::state_tracker_client::StateTrackerClient;
use std::sync::Arc;

pub struct DefinitionRequestDispatch {
    request_receiver: Receiver<DefinitionStorageAction>,
    definition_provider_api: AmqpWrapper,

    api_consumer: Arc<ApiConsumer>,

    identity: Arc<Identity>,
    token: Arc<Token>,
    token_expiration_margin_in_seconds: u64,

    state_tracker_client: StateTrackerClient,
}

impl DefinitionRequestDispatch {
    pub async fn try_new(
        id: String,
        request_receiver: Receiver<DefinitionStorageAction>,
        config: ConsumerConfig,
        api_consumer: Arc<ApiConsumer>,
        identity: Arc<Identity>,
        mut state_tracker_client: StateTrackerClient,
    ) -> Result<DefinitionRequestDispatch, Error> {
        let token_expiration_margin_in_seconds = config.token_expiration_margin_in_seconds();

        let amqp_wrapper = match AmqpWrapper::try_new(config.owned_amqp_connect_config()) {
            Ok(amqp_wrapper) => amqp_wrapper,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to initialize amqp wrapper: {}", error.message),
                ))
            }
        };

        let token = match identity.try_get_token().await {
            Ok(token) => token,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!("failed to get token: {}", error),
                ))
            }
        };

        state_tracker_client.set_id(id);

        Ok(DefinitionRequestDispatch {
            request_receiver,
            definition_provider_api: amqp_wrapper,
            api_consumer,
            identity,
            token,
            token_expiration_margin_in_seconds,
            state_tracker_client,
        })
    }

    pub async fn run(mut self) -> Result<(), Error> {
        let channel = match self.definition_provider_api.try_get_channel().await {
            Ok(channel) => channel,
            Err(error) => {
                return Err(Error::new(
                    ErrorKind::InternalFailure,
                    format!(
                        "failed to get channel from connection to definition provider: {}",
                        error
                    ),
                ));
            }
        };

        loop {
            let token_result = match self
                .identity
                .renew_token_if_expiring_after_seconds(
                    self.token.clone(),
                    self.token_expiration_margin_in_seconds,
                )
                .await
            {
                Ok(token) => {
                    self.token = token;

                    Ok(())
                }
                Err(error) => Err(Error::new(
                    ErrorKind::ExternalFailure,
                    format!("failed to get token: {}", error),
                )),
            };

            let request = match self.request_receiver.recv().await {
                Ok(request) => request,
                Err(error) => {
                    let error_message = format!("failed to receive request: {}", error);

                    match self
                        .state_tracker_client
                        .send_state(State::Error(error_message.clone()))
                        .await
                    {
                        Ok(_) => (),
                        Err(error) => log::warn!("failed to send state: {}", error),
                    }

                    log::error!("{}", error_message);
                    continue;
                }
            };

            let api_consumer = self.api_consumer.clone();

            match request {
                DefinitionStorageAction::FindByVersion { version, replier } => {
                    // This action requires authentication through a token.
                    // Therefore, no token, no action.
                    match token_result {
                        Ok(_) => (),
                        Err(error) => {
                            let error_message = format!("failed to renew token: {}", error);

                            match self
                                .state_tracker_client
                                .send_state(State::Error(error_message.clone()))
                                .await
                            {
                                Ok(_) => (),
                                Err(error) => log::warn!("failed to send state: {}", error),
                            }

                            log::error!("{}", error_message);

                            match replier.send(Err(cooplan_amqp_api_consumer::error::Error::new(
                                cooplan_amqp_api_consumer::error::ErrorKind::InternalFailure,
                                error_message,
                            ))) {
                                Ok(_) => (),
                                Err(_) => log::error!("failed to send error to replier"),
                            }

                            continue;
                        }
                    }

                    tokio::spawn(elements::amqp_definition::find_by_version(
                        version,
                        channel.clone(),
                        self.token.value().to_string(),
                        api_consumer,
                        replier,
                    ));
                }
                DefinitionStorageAction::FindLatest { replier } => {
                    tokio::spawn(elements::amqp_definition::find_latest(
                        channel.clone(),
                        api_consumer,
                        replier,
                    ));
                }
            }

            match self.state_tracker_client.send_state(State::Valid).await {
                Ok(_) => (),
                Err(error) => log::warn!("failed to send state: {}", error),
            }
        }
    }
}
