use cooplan_definitions_lib::definition::Definition;
use tokio::sync::watch::Sender;

pub struct DefinitionProviderConsumer {
    sender: Sender<Definition>,
}

impl DefinitionProviderConsumer {}
