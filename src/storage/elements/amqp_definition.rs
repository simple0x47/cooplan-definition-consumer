use cooplan_amqp_api_consumer::error::Error;
use cooplan_amqp_api_consumer::output::read_from_stream;
use cooplan_amqp_api_consumer::rpc::send_request_and_wait_for_response;
use cooplan_definitions_lib::definition::Definition;
use cooplan_lapin_wrapper::config::api_consumer::ApiConsumer;
use lapin::Channel;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

pub async fn find_by_version(
    version: String,
    channel: Arc<Channel>,
    token: String,
    api_consumer: Arc<ApiConsumer>,
    replier: Sender<Result<Definition, Error>>,
) -> Result<(), Error> {
    const API_CONSUMER_INPUT_ID: &str = "definition_by_version";

    let request = find_by_version_request(&version, &token);

    send_request_and_wait_for_response::<Definition>(
        api_consumer,
        API_CONSUMER_INPUT_ID,
        channel,
        request,
        replier,
    )
    .await?;

    Ok(())
}

fn find_by_version_request(version: &str, token: &str) -> Vec<u8> {
    let json_request = format!(
        "{{\
            \"header\": {{
                \"element\": \"definition\",
                \"action\": \"get\",
                \"token\": \"{}\" 
            }},
            \"version\": \"{}\"
        }}",
        token, version
    );

    json_request.into_bytes()
}

pub async fn find_latest(
    channel: Arc<Channel>,
    api_consumer: Arc<ApiConsumer>,
    replier: Sender<Result<Definition, Error>>,
) -> Result<(), Error> {
    const API_CONSUMER_OUTPUT_ID: &str = "latest_definition";

    read_from_stream(api_consumer, API_CONSUMER_OUTPUT_ID, channel, replier).await?;

    Ok(())
}
