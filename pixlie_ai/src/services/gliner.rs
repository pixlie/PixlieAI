// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::config::Settings;
use crate::entity::named_entity::EntityName;
use crate::entity::named_entity::ExtractedEntity;
use crate::error::{PiError, PiResult};
use gliner::model::pipeline::token::TokenMode;
use gliner::model::{input::text::TextInput, params::Parameters, GLiNER};
use orp::params::RuntimeParameters;
use std::path::PathBuf;
use std::str::FromStr;

pub fn extract_entities(text: String, labels: Vec<EntityName>) -> PiResult<Vec<ExtractedEntity>> {
    let settings: Settings = Settings::get_cli_settings()?;
    let path_to_storage_dir = match settings.path_to_storage_dir {
        Some(path) => PathBuf::from(path),
        None => {
            return Err(PiError::InternalError(
                "Cannot find path to storage directory".to_string(),
            ));
        }
    };

    let model = match GLiNER::<TokenMode>::new(
        Parameters::default(),
        RuntimeParameters::default(),
        path_to_storage_dir.join("gliner_onnx_models/multitask_large_v0_5/tokenizer.json"),
        path_to_storage_dir.join("gliner_onnx_models/multitask_large_v0_5/model.onnx"),
    ) {
        Ok(model) => model,
        Err(err) => {
            return Err(PiError::GlinerError(err.to_string()));
        }
    };

    let input = match TextInput::new(vec![text], labels.iter().map(|x| x.to_string()).collect()) {
        Ok(input) => input,
        Err(err) => {
            return Err(PiError::GlinerError(err.to_string()));
        }
    };

    let output = match model.inference(input) {
        Ok(output) => output,
        Err(err) => {
            return Err(PiError::GlinerError(err.to_string()));
        }
    };

    let mut extracted_entities: Vec<ExtractedEntity> = vec![];
    for spans in output.spans {
        for span in spans {
            let entity_name = match EntityName::from_str(span.class()) {
                Ok(entity_name) => entity_name,
                Err(_) => {
                    continue;
                }
            };
            extracted_entities.push(ExtractedEntity {
                entity_name,
                matching_text: span.text().to_string(),
                start: Some(span.offsets().0),
                end: Some(span.offsets().1),
                probability: Some(span.probability()),
            })
        }
    }

    Ok(extracted_entities)
}
