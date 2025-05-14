// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::entity::ExtractedEntity;
use serde::Serialize;

pub mod anthropic;
pub mod gliner;
pub mod llm_provider;
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
