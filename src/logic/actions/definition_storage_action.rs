use cooplan_amqp_api_consumer::error::Error;
use cooplan_definitions_lib::definition::Definition;
use tokio::sync::oneshot::Sender;

pub enum DefinitionStorageAction {
    FindByVersion {
        version: String,
        replier: Sender<Result<Definition, Error>>,
    },
    FindLatest {
        replier: Sender<Result<Definition, Error>>,
    },
}
