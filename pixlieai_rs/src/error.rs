// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PiError {
    #[error("Cannot read config file")]
    CannotReadConfigFile,

    #[error("Config error: {0}")]
    SettingsError(#[from] config::ConfigError),

    #[error("Faile to write config file")]
    FailedToWriteConfigFile(String),

    #[error("API key not configured")]
    ApiKeyNotConfigured,

    // #[error("Error from Python code: {0}")]
    // PythonError(#[from] pyo3::PyErr),
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
}

pub type PiResult<T> = Result<T, PiError>;
