use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Debug, Deserialize, Serialize)]
pub enum LibDwataError {
    // RocksDB for Dwata
    CouldNotCreateDwataDB,
    CouldNotConnectToDwataDB,
    CouldNotInsertToDwataDB,
    CouldNotUpdateDwataDB,
    CouldNotFetchDataFromDwataDB,
    CouldNotMigrateDwataDB,

    // AI providers/models/features
    InvalidAIModel,
    CouldNotConnectToAIProvider,
    CouldNotGenerateEmbedding,
    FeatureNotAvailableWithAIProvider,
    InvalidProcessStatus,

    // Chat and its processing related
    ChatHasNoMessage,
    NoRequestedAIModel,
    BeingProcessedByAI,
    AlreadyProcessedByAI,
    ChatHasNoRootId,
    ToolUseNotSupported,

    // Integrated vector DB
    CouldNotConnectToVectorDB,

    // Chat context related
    CouldNotFindNode,

    // Directory related
    CouldNotOpenDirectory,

    // Task and app state related
    InvalidTaskStatus,
    TaskHasRunRecently,
    AppStateNotFound,

    // API requests related
    CouldNotConnectToAPI,

    // Enum related
    CouldNotParseEnum,

    // Search related
    CouldNotCreateSearchIndex,
    SearchIndexDoesNotExist,
    CouldNotOpenSearchIndex,
    CouldNotParseSearchQuery,
    CouldNotSearchTheIndex,
}

impl Error for LibDwataError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

impl std::fmt::Display for LibDwataError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
