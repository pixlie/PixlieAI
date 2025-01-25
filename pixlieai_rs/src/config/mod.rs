// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::{
    error::{PiError, PiResult},
    services::{EntityExtractionProvider, TextClassificationProvider},
};
use bytes::Buf;
use config::Config;
use dirs::config_dir;
use flate2::read::GzDecoder;
use gliner::get_is_gliner_setup;
use log::{debug, error};
use python::check_system_python;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, exists, File},
    io::Write,
    path::PathBuf,
};
use tar::Archive;
use ts_rs::TS;

pub mod api;
pub mod gliner;
pub mod python;

#[derive(Deserialize, Serialize, TS)]
#[ts(export, rename_all = "camelCase")]
pub struct Settings {
    pub anthropic_api_key: Option<String>,
    pub ollama_hosts: Option<Vec<String>>,
    pub ollama_port: Option<u16>,
    pub gpu_hosts: Option<Vec<String>>,
    pub path_to_storage_dir: Option<String>,
    pub current_project: Option<String>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub enum SettingsIncompleteReason {
    MissingLLMProvider,
    StorageDirNotConfigured,
    PythonNotAvailable,
    PythonVenvNotAvailable,
    PythonPipNotAvailable,
    GlinerNotSetup,
}

#[derive(Serialize, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum SettingsStatus {
    Incomplete(Vec<SettingsIncompleteReason>),
    Complete,
}

pub fn check_cli_settings() -> PiResult<()> {
    let config_path = config_dir();
    if config_path.is_none() {
        error!("Can not detect the config directory of the current user");
        return Err(PiError::CannotDetectConfigDirectory);
    }
    let mut config_path = config_path.unwrap();
    config_path.push("pixlie_ai");
    if !config_path.exists() {
        // Create the `pixlie_ai` config directory since it does not exist
        match create_dir(config_path.clone()) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Could not create config directory at {}\nError: {}",
                    config_path.display(),
                    err
                );
                return Err(PiError::CannotReadOrWriteToConfigDirectory);
            }
        }
    };
    config_path.push("settings.toml");
    if config_path.exists() {
        match config_path.to_str() {
            Some(config_path) => {
                match Config::builder()
                    .add_source(config::File::with_name(config_path))
                    .build()
                {
                    Ok(_) => {}
                    Err(err) => {
                        error!(
                            "Could not load config file at {}\nError: {}",
                            config_path, err
                        );
                        return Err(PiError::CannotReadOrWriteToConfigDirectory);
                    }
                }
            }
            None => {
                return Err(PiError::CannotDetectConfigDirectory);
            }
        }
    };
    Ok(())
}

pub fn get_cli_settings_path() -> PiResult<(PathBuf, PathBuf)> {
    let mut path_to_config_dir = config_dir().unwrap();
    path_to_config_dir.push("pixlie_ai");
    let mut path_to_config_file = path_to_config_dir.clone();
    path_to_config_file.push("settings.toml");
    // Create a blank config file if it does not exist
    if !path_to_config_file.exists() {
        let mut config_file = File::create(path_to_config_file.clone())?;
        config_file.write_all(b"")?;
    }
    debug!("CLI settings path {}", path_to_config_file.display());
    Ok((path_to_config_dir, path_to_config_file))
}

pub fn download_admin_site(admin_path: &PathBuf) -> PiResult<()> {
    // We download admin.tar.gz from our GitHub release
    let admin_tar_gz_url =
        "https://github.com/pixlie/PixlieAI/releases/download/v0.1.0/admin.tar.gz";
    let admin_tar_gz_response = reqwest::blocking::get(admin_tar_gz_url)?;
    let admin_tar_gz_bytes = admin_tar_gz_response.bytes()?;
    // Use flate2 to decompress the tar.gz file
    let admin_tar_gz = GzDecoder::new(admin_tar_gz_bytes.reader());
    // Use tar to extract the files from the tar.gz file
    Archive::new(admin_tar_gz).unpack(&admin_path)?;
    Ok(())
}

pub fn get_static_admin_dir() -> PiResult<PathBuf> {
    let (path_to_config_dir, _path_to_config_file) = get_cli_settings_path()?;
    let mut static_root = PathBuf::from(path_to_config_dir.clone());
    static_root.push("admin");
    // Create the `admin` directory if it does not exist
    match exists(&static_root) {
        Ok(true) => {}
        Ok(false) => match create_dir(static_root.clone()) {
            Ok(_) => {
                download_admin_site(&static_root)?;
            }
            Err(err) => {
                error!(
                    "Could not create admin directory at {}\nError: {}",
                    static_root.display(),
                    err
                );
                return Err(PiError::CannotReadOrWriteToConfigDirectory);
            }
        },
        Err(err) => {
            error!(
                "Could not check if admin directory exists at {}\nError: {}",
                static_root.display(),
                err
            );
            return Err(PiError::CannotReadOrWriteToConfigDirectory);
        }
    }
    Ok(static_root)
}

impl Settings {
    pub fn get_cli_settings() -> PiResult<Self> {
        let (_path_to_config_dir, path_to_config_file) = get_cli_settings_path()?;
        match path_to_config_file.to_str() {
            Some(config_path) => {
                let settings = Config::builder()
                    .add_source(config::File::with_name(config_path))
                    .build()?;
                let mut settings = settings.try_deserialize::<Settings>()?;
                settings.ollama_port = Some(settings.ollama_port.unwrap_or(8080));
                Ok(settings)
            }
            None => Err(PiError::CannotReadOrWriteConfigFile),
        }
    }

    pub fn get_settings_status(&self) -> PiResult<SettingsStatus> {
        let mut incomplete_reasons = Vec::new();
        if self.path_to_storage_dir.is_none() {
            incomplete_reasons.push(SettingsIncompleteReason::StorageDirNotConfigured);
        }
        let python_status = check_system_python();
        if python_status.is_none() {
            incomplete_reasons.push(SettingsIncompleteReason::PythonNotAvailable);
        } else {
            let python_status = python_status.unwrap();
            if !python_status.venv {
                incomplete_reasons.push(SettingsIncompleteReason::PythonVenvNotAvailable);
            } else if !python_status.pip {
                incomplete_reasons.push(SettingsIncompleteReason::PythonPipNotAvailable);
            } else if !get_is_gliner_setup()? {
                incomplete_reasons.push(SettingsIncompleteReason::GlinerNotSetup);
            }
        }
        if self.anthropic_api_key.is_none() && self.ollama_hosts.is_none() {
            incomplete_reasons.push(SettingsIncompleteReason::MissingLLMProvider);
        }
        if incomplete_reasons.is_empty() {
            Ok(SettingsStatus::Complete)
        } else {
            Ok(SettingsStatus::Incomplete(incomplete_reasons))
        }
    }

    pub fn get_entity_extraction_provider(&self) -> PiResult<EntityExtractionProvider> {
        if let true = get_is_gliner_setup()? {
            return Ok(EntityExtractionProvider::Gliner);
        } else if let Some(_) = self.ollama_hosts {
            return Ok(EntityExtractionProvider::Ollama);
        } else if let Some(_) = self.anthropic_api_key {
            return Ok(EntityExtractionProvider::Anthropic);
        }
        Err(PiError::NotConfiguredProperly)
    }

    pub fn get_text_classification_provider(&self) -> PiResult<TextClassificationProvider> {
        if let Some(_) = self.ollama_hosts {
            return Ok(TextClassificationProvider::Ollama);
        } else if let Some(_) = self.anthropic_api_key {
            return Ok(TextClassificationProvider::Anthropic);
        }
        Err(PiError::NotConfiguredProperly)
    }

    pub fn merge_updates(&mut self, updates: &Settings) {
        if updates.anthropic_api_key.is_some() {
            self.anthropic_api_key = updates.anthropic_api_key.clone();
        }
        if updates.ollama_hosts.is_some() {
            self.ollama_hosts = updates.ollama_hosts.clone();
        }
        if updates.ollama_port.is_some() {
            self.ollama_port = updates.ollama_port.clone();
        }
        if updates.path_to_storage_dir.is_some() {
            self.path_to_storage_dir = updates.path_to_storage_dir.clone();
        }
        if updates.current_project.is_some() {
            self.current_project = updates.current_project.clone();
        }
    }

    pub fn write_to_config_file(&self) -> PiResult<()> {
        let (_path_to_config_dir, path_to_config_file) = get_cli_settings_path()?;
        // Write the TOML file to the config file
        match toml::to_string_pretty(self) {
            Ok(config_string) => {
                let mut config_file = File::create(path_to_config_file)?;
                config_file.write_all(config_string.as_bytes())?;
                Ok(())
            }
            Err(err) => Err(PiError::FailedToWriteConfigFile(err.to_string())),
        }
    }
}
