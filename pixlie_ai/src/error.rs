// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::PiEvent;
use actix_web::ResponseError;
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

    #[error("Config error: {0}")]
    SettingsError(#[from] config::ConfigError),

    #[error("Failed to write config file: {0}")]
    FailedToWriteConfigFile(String),

    #[error("API key not configured")]
    ApiKeyNotConfigured,

    #[error("Error from reqwest: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("Failed to fetch after retries")]
    FetchFailedAfterRetries,

    #[error("Not configured properly")]
    NotConfiguredProperly,

    #[error("Could not classify text")]
    CouldNotClassifyText,

    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("Error sending to crossbeam channel: {0}")]
    CrossbeamChannelError(#[from] crossbeam_channel::SendError<PiEvent>),

    #[error("Error from postcard: {0}")]
    PostcardError(#[from] postcard::Error),

    #[error("Error from rocksdb: {0}")]
    RocksdbError(#[from] rocksdb::Error),

    #[error("Error from Actix Web Blocking Error: {0}")]
    ActixWebError(#[from] actix_web::error::BlockingError),

    #[error("Error in CRUD: {0}")]
    CrudError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ResponseError for PiError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            // PiError::CannotReadConfigFile => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // PiError::SettingsError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // PiError::FailedToWriteConfigFile(_) => {
            //     actix_web::http::StatusCode::INTERNAL_SERVER_ERROR
            // }
            // PiError::ApiKeyNotConfigured => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // PiError::ReqwestError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // PiError::FetchFailedAfterRetries => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // PiError::NotConfiguredProperly => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // PiError::CouldNotClassifyText => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            // PiError::IOError(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

pub type PiResult<T> = Result<T, PiError>;
