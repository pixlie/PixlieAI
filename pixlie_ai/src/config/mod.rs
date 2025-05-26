// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::{
    error::{PiError, PiResult},
    PIXLIE_VERSION_NUMBER,
};
use bytes::Buf;
use config::Config;
use dirs::config_dir;
use flate2::read::GzDecoder;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, exists, remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};
use tar::Archive;
use ts_rs::TS;
use utoipa::ToSchema;

pub mod api;
pub mod gliner;
pub mod python;

#[derive(Debug, Deserialize, Serialize, ToSchema, TS)]
#[ts(export, rename_all = "camelCase")]
pub struct Settings {
    pub path_to_storage_dir: Option<String>,
    // When the hostname is set, we look for the `Certs/<hostname>/` directory in the storage directory
    pub hostname: Option<String>,
}

pub struct WithHostname {
    pub hostname: String,
    pub path_to_certificate: PathBuf,
    pub path_to_key: PathBuf,
}

#[derive(Serialize, ToSchema, TS)]
#[ts(export)]
pub enum SettingsIncompleteReason {
    StorageDirNotConfigured,
    GlinerDirNotFound,
    GlinerFileNotFound,
}

#[derive(Serialize, ToSchema, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum SettingsStatus {
    Incomplete(Vec<SettingsIncompleteReason>),
    Complete,
}

pub fn check_cli_settings() -> PiResult<()> {
    let mut config_path = match config_dir() {
        Some(config_path) => config_path,
        None => {
            error!("Can not detect the config directory of the current user");
            return Err(PiError::CannotDetectConfigDirectory);
        }
    };
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

// TODO: Move and refactor this logic to impl Settings::new,
// eliminate the need for get_cli_settings_path
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
    Ok((path_to_config_dir, path_to_config_file))
}

// TODO: Move this function to impl Settings, eliminate the need for
// get_cli_settings_path
pub fn download_admin_site() -> PiResult<()> {
    // We download admin.tar.gz from our GitHub release
    let static_admin_dir = get_static_admin_dir()?;
    let mut static_index_html = static_admin_dir.clone();
    static_index_html.push("index.html");

    match exists(&static_admin_dir) {
        Ok(true) => {
            info!("Admin site already exists, removing it");
            remove_dir_all(static_admin_dir.clone())?;
        }
        _ => {}
    }

    info!("Downloading admin site from GitHub");
    let admin_tar_gz_url = format!(
        "https://github.com/pixlie/PixlieAI/releases/download/v{}/admin-v{}.tar.gz",
        PIXLIE_VERSION_NUMBER, PIXLIE_VERSION_NUMBER,
    );
    // Download the admin.tar.gz file
    match reqwest::blocking::get(&admin_tar_gz_url) {
        Ok(response) => {
            if response.status() != 200 {
                return Err(PiError::FetchError(format!(
                    "Could not download admin.tar.gz from {}\nStatus: {}",
                    admin_tar_gz_url,
                    response.status()
                )));
            } else {
                // Save the response to a file
                let admin_tar_gz_bytes = response.bytes()?;
                // Use flate2 to decompress the tar.gz file
                let admin_tar_gz = GzDecoder::new(admin_tar_gz_bytes.reader());
                create_dir(static_admin_dir.clone())?;
                // Use tar to extract the files from the tar.gz file
                Archive::new(admin_tar_gz).unpack(&static_admin_dir)?;
                // Check if index.html exists in unpacked files
                match exists(&static_index_html) {
                    Ok(_) => {}
                    Err(err) => {
                        remove_dir_all(static_admin_dir.clone())?;
                        return Err(PiError::FetchError(
                            format!(
                                "Could not find index.html in downloaded admin from {}, Error: {}\nRemoving admin directory",
                                admin_tar_gz_url,
                                err
                            ),
                        ));
                    }
                }
            }
        }
        Err(err) => {
            return Err(PiError::FetchError(format!(
                "Could not download admin.tar.gz from {}\nError: {}",
                admin_tar_gz_url, err
            )));
        }
    };
    Ok(())
}

// TODO: Move this function to impl Settings, eliminate the need for
// get_cli_settings_path
pub fn get_static_admin_dir() -> PiResult<PathBuf> {
    let (path_to_config_dir, _path_to_config_file) = get_cli_settings_path()?;
    let mut static_root = PathBuf::from(path_to_config_dir.clone());
    static_root.push("admin");
    Ok(static_root)
}

impl Settings {
    pub fn get_cli_settings() -> PiResult<Self> {
        let (_path_to_config_dir, path_to_config_file) = get_cli_settings_path()?;
        match path_to_config_file.to_str() {
            Some(config_path) => {
                let settings = match Config::builder()
                    .add_source(config::File::with_name(config_path))
                    .build()
                {
                    Ok(settings) => settings,
                    Err(err) => {
                        error!("Error reading settings: {}", err);
                        return Err(PiError::CannotReadOrWriteConfigFile);
                    }
                };
                let settings = match settings.try_deserialize::<Settings>() {
                    Ok(settings) => settings,
                    Err(err) => {
                        error!("Error deserializing settings: {}", err);
                        return Err(PiError::CannotReadOrWriteConfigFile);
                    }
                };
                Ok(settings)
            }
            None => {
                error!("Cannot find config file");
                Err(PiError::CannotReadOrWriteConfigFile)
            }
        }
    }

    pub fn get_settings_status(&self) -> PiResult<SettingsStatus> {
        let mut incomplete_reasons = Vec::new();
        match &self.path_to_storage_dir {
            Some(path_str) => {
                let storage_dir_path = Path::new(path_str);
                let gliner_dir_path =
                    storage_dir_path.join("gliner_onnx_models/multitask_large_v0_5");
                if !gliner_dir_path.exists() || !gliner_dir_path.is_dir() {
                    incomplete_reasons.push(SettingsIncompleteReason::GlinerDirNotFound);
                } else {
                    let model_file_path = gliner_dir_path.join("model.onnx");
                    let tokenizer_file_path = gliner_dir_path.join("tokenizer.json");
                    if !model_file_path.exists() || !tokenizer_file_path.exists() {
                        incomplete_reasons.push(SettingsIncompleteReason::GlinerFileNotFound);
                    }
                }
            }
            None => {
                incomplete_reasons.push(SettingsIncompleteReason::StorageDirNotConfigured);
            }
        }
        if incomplete_reasons.is_empty() {
            Ok(SettingsStatus::Complete)
        } else {
            Ok(SettingsStatus::Incomplete(incomplete_reasons))
        }
    }

    pub fn merge_updates(&mut self, updates: &Settings) {
        if updates.path_to_storage_dir.is_some() {
            self.path_to_storage_dir = updates.path_to_storage_dir.clone();
        }
        if updates.hostname.is_some() {
            self.hostname = updates.hostname.clone();
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

    pub fn get_hostname(&self) -> PiResult<Option<WithHostname>> {
        // Hostname is needed to run the API server on a specific hostname
        // Since this is needed before we can configure anything else,
        // we store hostname and certificates in user's config directory
        // instead of the storage directory
        let path_to_hostname_storage_dir: PathBuf = match self.hostname {
            Some(ref hostname) => match config_dir() {
                Some(config_path) => config_path.join("pixlie_ai").join(hostname),
                None => {
                    error!("Can not detect the config directory of the current user");
                    return Err(PiError::InternalError(
                        "Can not detect the config directory of the current user".to_string(),
                    ));
                }
            },
            None => {
                return Ok(None);
            }
        };
        Ok(Some(WithHostname {
            hostname: self.hostname.clone().unwrap(),
            path_to_certificate: path_to_hostname_storage_dir.join("Certificates/cert.pem"),
            path_to_key: path_to_hostname_storage_dir.join("Certificates/key.pem"),
        }))
    }
}
