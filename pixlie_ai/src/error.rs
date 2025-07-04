// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::PiEvent;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PiError {
    #[error("Cannot detect the config directory of the current user")]
    CannotDetectConfigDirectory,

    #[error("Cannot read or write to config directory")]
    CannotReadOrWriteToConfigDirectory,

    #[error("Cannot read config file")]
    CannotReadOrWriteConfigFile,

    #[error("Cannot read or write to storage directory")]
    CannotReadOrWriteToStorageDirectory,

    #[error("Failed to write config file: {0}")]
    FailedToWriteConfigFile(String),

    #[error("API key for {0} not configured")]
    ApiKeyNotConfigured(String),

    #[error("Could not parse response from LLM: {0}")]
    CouldNotParseResponseFromLLM(String),

    #[error("Error in fetching external data: {0}")]
    FetchError(String),

    #[error("Internal:{0}")]
    InternalError(String),

    #[error("Version check error: {0}")]
    VersionCheckError(String),

    #[error("Version mismatch: Cargo version: {0}, Expected version: {1}")]
    VersionMismatch(String, String),

    #[error("Version file missing")]
    VersionFileMissing,

    #[error("Error in graph reading or writing: {0}")]
    GraphError(String),

    #[error("Feature is not available: {0}")]
    FeatureNotAvailable(String),

    #[error("CRUD Error{0:?}: {1}")]
    CrudError(Vec<String>, String),

    #[error("{0} {1} not found")]
    CrudNotFoundError(String, String),

    #[error("Gliner: {0}")]
    GlinerError(String),

    #[error("IO: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Reqwest: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Config: {0}")]
    SettingsError(#[from] config::ConfigError),

    #[error("Crossbeam: {0}")]
    CrossbeamChannelError(#[from] crossbeam_channel::SendError<PiEvent>),

    #[error("Postcard: {0}")]
    PostcardError(#[from] postcard::Error),

    #[error("Rocksdb: {0}")]
    RocksdbError(#[from] rocksdb::Error),

    #[error("Could not parse NodeLabel from string: {0}")]
    CouldNotParseNodeLabel(#[from] strum::ParseError),

    #[error("Could not generate TypeScript schema: {0}")]
    CouldNotGenerateTypeScriptSchema(#[from] ts_rs::ExportError),

    #[error("Error in serde_json: {0}")]
    SerdeJsonError(#[from] serde_json::Error),

    #[error("Error in URL parsing: {0}")]
    UrlParseError(#[from] url::ParseError),
}

impl actix_web::ResponseError for PiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type PiResult<T> = Result<T, PiError>;
