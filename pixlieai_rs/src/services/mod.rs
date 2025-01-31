// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::entity::ExtractedEntity;
use serde::Serialize;

pub mod anthropic;
// pub mod gliner;
pub mod ollama;

pub enum EntityExtractionProvider {
    Anthropic,
    Gliner,
    Ollama,
}

pub enum TextClassificationProvider {
    Anthropic,
    Ollama,
}

pub struct EntityExtractionExample {
    pub text: String,
    pub extractions: Vec<ExtractedEntity>,
}

#[derive(Serialize)]
pub struct ExtractionRequest {
    pub text: String,
    pub labels: Vec<String>,
}

pub fn extract_entites_from_lines(lines: &str) -> Vec<ExtractedEntity> {
    // This function is mainly used to extract entities from API responses from large language models
    // Each line (in the CSV format) is an entity type and the matching text

    let mut extracted: Vec<ExtractedEntity> = vec![];
    let text = lines;
    let mut reader = csv::Reader::from_reader(text.as_bytes());
    for line in reader.records() {
        match line {
            Ok(line) => {
                extracted.push(ExtractedEntity {
                    label: line.get(0).unwrap().to_string(),
                    matching_text: line.get(1).unwrap().to_string(),
                    ..Default::default()
                });
            }
            Err(_) => {}
        }
    }
    extracted
}
