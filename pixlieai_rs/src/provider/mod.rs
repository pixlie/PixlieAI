// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::entity::{EntityType, ExtractedEntity};
use serde::Serialize;
use std::{fmt::Display, hash::Hasher, str::FromStr};
use strum::EnumString;

pub mod anthropic;
pub mod gliner;

#[derive(Serialize)]
pub enum LLMProvider {
    Anthropic,
}

#[derive(Serialize)]
pub struct LargeLanguageModel {
    pub label: String,
    pub ai_provider: LLMProvider,
    pub api_name: String,

    pub context_window: Option<u32>,

    // Prices are in US cents
    pub price_per_million_input_tokens: Option<i32>,
    pub price_per_million_output_tokens: Option<i32>,
}

pub trait EntityExtraction {
    // fn get_payload_content_type(&self) -> String;

    fn get_labels(&self) -> Vec<EntityType>;

    // fn get_example_text(&self) -> String {
    //     String::from("")
    // }

    // fn get_example_extractions(&self) -> Vec<(EntityType, String)> {
    //     vec![]
    // }

    fn get_payload(&self) -> String;
}

pub fn extract_entites_from_lines(lines: &str) -> Vec<ExtractedEntity> {
    let mut extracted: Vec<ExtractedEntity> = vec![];
    let text = lines;
    let mut reader = csv::Reader::from_reader(text.as_bytes());
    for line in reader.records() {
        match line {
            Ok(line) => {
                extracted.push(ExtractedEntity {
                    entity_type: EntityType::try_from(line.get(0).unwrap().to_string().as_str())
                        .unwrap(),
                    matching_text: line.get(1).unwrap().to_string(),
                });
            }
            Err(_) => {}
        }
    }
    extracted
}
