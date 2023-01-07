use crate::error::Error;
use cooplan_definitions_lib::definition::Definition;
use lapin::Channel;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;

pub async fn find_by_version(
    version: String,
    channel: Arc<Channel>,
    token: String,
    replier: Sender<Result<Definition, Error>>,
) -> Result<(), Error> {
    let request = find_by_version_request(&version, &token);

    const AMQP_QUEUE_NAME: &str = "definition";

    Ok(())
}

fn find_by_version_request(version: &str, token: &str) -> String {
    format!(
        "{{\
            \"header\": {{
                \"element\": \"definition\",
                \"action\": \"get\"
                \"token\": \"{}\" 
            }},
            \"version\": \"{}\"
        }}",
        token, version
    )
}

pub async fn find_latest(
    channel: Arc<Channel>,
    replier: Sender<Result<Definition, Error>>,
) -> Result<(), Error> {
    const AMQP_QUEUE_NAME: &str = "latest_definition";

    Ok(())
}
