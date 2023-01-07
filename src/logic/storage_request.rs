use crate::logic::actions::definition_storage_action::DefinitionStorageAction;

pub enum StorageRequest {
    DefinitionRequest(DefinitionStorageAction),
}
