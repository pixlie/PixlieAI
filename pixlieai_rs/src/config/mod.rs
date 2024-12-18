// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::{
    engine::{Engine, Payload},
    entity::web::Link,
    error::{PiError, PiResult},
    services::{EntityExtractionProvider, TextClassificationProvider},
};
use config::Config;
use dirs::config_dir;
use log::{debug, error};
use python::check_system_python;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, create_dir_all, File},
    io::Write,
    path::PathBuf,
};
use ts_rs::TS;

pub mod gliner;
pub mod mqtt;
pub mod python;

#[derive(Deserialize, Serialize, TS)]
#[ts(export, rename_all = "camelCase")]
pub struct Settings {
    pub anthropic_api_key: Option<String>,
    pub ollama_hosts: Option<Vec<String>>,
    pub ollama_port: Option<u16>,
    pub mqtt_broker_host: Option<String>,
    pub path_to_storage_dir: Option<String>,
    pub current_project: Option<String>,
}

#[derive(Serialize, TS)]
#[ts(export)]
pub enum SettingsIncompleteReason {
    MissingLLMProvider,
    MissingMqtt,
    StorageDirNotConfigured,
    PythonNotAvailable,
    PythonVenvNotAvailable,
    PythonPipNotAvailable,
    MissingGliner,
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
        error!("Could not detect the config directory");
        return Err(PiError::CannotReadConfigFile);
    }
    let mut config_path = config_path.unwrap();
    config_path.push("pixlie_ai");
    let mut static_root = config_path.clone();
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
                return Err(PiError::CannotReadConfigFile);
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
                        return Err(PiError::CannotReadConfigFile);
                    }
                }
            }
            None => {
                return Err(PiError::CannotReadConfigFile);
            }
        }
    };
    static_root.push("PixlieAI");
    static_root.push("admin");
    static_root.push("dist");
    if !static_root.exists() {
        // Create the `pixlie_ai` config directory since it does not exist
        match create_dir_all(static_root.clone()) {
            Ok(_) => {}
            Err(err) => {
                error!(
                    "Could not create static directory at {}\nError: {}",
                    &static_root.display(),
                    err
                );
                return Err(PiError::CannotReadConfigFile);
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
    debug!("CLI settings path {}", path_to_config_file.display());
    Ok((path_to_config_dir, path_to_config_file))
}

pub fn get_path_to_static_dir() -> PiResult<PathBuf> {
    let (path_to_config_dir, _path_to_config_file) = get_cli_settings_path()?;
    let mut static_root = PathBuf::from(path_to_config_dir.clone());
    static_root.push("PixlieAI");
    static_root.push("admin");
    static_root.push("dist");
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
            None => Err(PiError::CannotReadConfigFile),
        }
    }

    pub fn get_is_gliner_available(&self) -> bool {
        // GLiNER is supported locally only
        // We check if the virtual environment for GLiNER has been created
        // The virtual environment is created in a gliner/.venv directory in the cli settings directory
        let (path_to_config_dir, _path_to_config_file) = get_cli_settings_path().unwrap();
        let mut path_to_gliner_venv = path_to_config_dir.clone();
        path_to_gliner_venv.push("gliner");
        path_to_gliner_venv.push(".venv");
        false
    }

    pub fn get_settings_status(&self) -> SettingsStatus {
        let mut incomplete_reasons = Vec::new();
        if self.anthropic_api_key.is_none() && self.ollama_hosts.is_none() {
            incomplete_reasons.push(SettingsIncompleteReason::MissingLLMProvider);
        }
        let python_status = check_system_python();
        if python_status.is_none() {
            incomplete_reasons.push(SettingsIncompleteReason::PythonNotAvailable);
        } else {
            let python_status = python_status.unwrap();
            if !python_status.venv {
                incomplete_reasons.push(SettingsIncompleteReason::PythonVenvNotAvailable);
            }
            if !python_status.pip {
                incomplete_reasons.push(SettingsIncompleteReason::PythonPipNotAvailable);
            }
        }
        if !self.get_is_gliner_available() {
            incomplete_reasons.push(SettingsIncompleteReason::MissingGliner);
        }
        if self.mqtt_broker_host.is_none() {
            incomplete_reasons.push(SettingsIncompleteReason::MissingMqtt);
        }
        if self.path_to_storage_dir.is_none() {
            incomplete_reasons.push(SettingsIncompleteReason::StorageDirNotConfigured);
        }
        if incomplete_reasons.is_empty() {
            SettingsStatus::Complete
        } else {
            SettingsStatus::Incomplete(incomplete_reasons)
        }
    }

    pub fn get_entity_extraction_provider(&self) -> PiResult<EntityExtractionProvider> {
        if let true = self.get_is_gliner_available() {
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
        if updates.mqtt_broker_host.is_some() {
            self.mqtt_broker_host = updates.mqtt_broker_host.clone();
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

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq)]
pub enum RuleCondition {
    IfContextIncludes(String),
}

#[derive(Deserialize, Serialize)]
pub struct Rule {
    pub applies_to: String,
    pub action: String,
    pub conditions: Vec<RuleCondition>,
}

impl Rule {
    pub fn new<S: Into<String>>(applies_to: S, action: S, conditions: Vec<RuleCondition>) -> Rule {
        Rule {
            applies_to: applies_to.into(),
            action: action.into(),
            conditions,
        }
    }
}

pub fn startup_funding_insights_app(engine: &mut Engine) {
    let data_extraction_conditions: Vec<RuleCondition> =
        ["Startup Funding", "Startup Investment", "Startup Product"]
            .iter()
            .map(|x| RuleCondition::IfContextIncludes(x.to_string()))
            .collect();
    let entity_extraction_conditions: Vec<RuleCondition> = [
        "Company",
        "Funding",
        "PreviousFunding",
        "TotalFunding",
        "Valuation",
        "FundingStage",
        "Investor",
        "Founder",
    ]
    .iter()
    .map(|x| RuleCondition::IfContextIncludes(x.to_string()))
    .collect();

    let link_extract_rule = Rule::new(
        "Link",
        "Extract a link to be crawled later if the following conditions are met",
        data_extraction_conditions.clone(),
    );
    let table_data_extract_rule = Rule::new(
        "Table",
        "Extract table data from the given table if the headings match the given conditions",
        data_extraction_conditions.clone(),
    );
    let entity_extract_rule = Rule::new(
        "Entity",
        "Extract entities from the given text if the following conditions are met",
        entity_extraction_conditions.clone(),
    );
    engine.add_node(Payload::Rule(link_extract_rule));
    engine.add_node(Payload::Rule(table_data_extract_rule));
    engine.add_node(Payload::Rule(entity_extract_rule));
    engine.add_node(Payload::Link(Link {
        url: "https://growthlist.co/funded-startups/".to_string(),
        text: "List of funded startups for 2024".to_string(),
        ..Default::default()
    }));
}
