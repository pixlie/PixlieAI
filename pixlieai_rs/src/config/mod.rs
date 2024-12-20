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
use log::error;
use serde::{Deserialize, Serialize};
use std::{
    fs::{create_dir, create_dir_all},
    path::PathBuf,
};

#[derive(Deserialize)]
pub struct Settings {
    pub anthropic_api_key: Option<String>,
    pub is_gliner_available: Option<bool>,
    pub ollama_hosts: Option<Vec<String>>,
    pub ollama_port: Option<u16>,
    pub mqtt_broker_host: Option<String>,
    #[serde(skip_deserializing)]
    pub path_to_config_dir: String,
    pub path_to_storage_dir: Option<String>,
    pub current_project: Option<String>,
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

pub fn get_cli_settings() -> PiResult<Settings> {
    let mut config_path = config_dir().unwrap();
    config_path.push("pixlie_ai");
    let path_to_config_dir = config_path.clone();
    config_path.push("settings.toml");
    match config_path.to_str() {
        Some(config_path) => {
            let settings = Config::builder()
                .add_source(config::File::with_name(config_path))
                .build()?;
            let mut settings = settings.try_deserialize::<Settings>()?;
            settings.ollama_port = Some(settings.ollama_port.unwrap_or(8080));
            settings.path_to_config_dir = path_to_config_dir.to_str().unwrap().to_string();
            Ok(settings)
        }
        None => Err(PiError::CannotReadConfigFile),
    }
}

impl Settings {
    pub fn get_entity_extraction_provider(&self) -> PiResult<EntityExtractionProvider> {
        if let Some(true) = self.is_gliner_available {
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

    pub fn get_path_to_static_dir(&self) -> PathBuf {
        let mut static_root = PathBuf::from(self.path_to_config_dir.clone());
        static_root.push("PixlieAI");
        static_root.push("admin");
        static_root.push("dist");
        static_root
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
