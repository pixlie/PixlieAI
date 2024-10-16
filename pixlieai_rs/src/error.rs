// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PiError {
    #[error("Config error: {0}")]
    SettingsError(#[from] config::ConfigError),

    #[error("Error from Python code: {0}")]
    PythonError(#[from] pyo3::PyErr),
}

pub type PiResult<T> = Result<T, PiError>;
