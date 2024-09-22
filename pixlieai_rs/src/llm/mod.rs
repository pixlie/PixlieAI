// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use serde::Serialize;

use crate::entity::EntityType;

pub mod anthropic;

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

pub trait EntityExtractionWithLLM {
    fn get_payload_content_type(&self) -> String;

    fn get_extractable_entity_types(&self) -> Vec<EntityType>;

    fn get_example_text(&self) -> String;

    fn get_example_extractions(&self) -> Vec<(EntityType, String)>;

    fn get_payload(&self) -> String;
}
