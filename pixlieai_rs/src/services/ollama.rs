// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::{
    entity::ExtractedEntity,
    error::{PiError, PiResult},
    services::extract_entites_from_lines,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct OllamaResponse {
    pub model: String,
    pub created_at: String,
    pub response: String,
    pub done: bool,
    pub context: Vec<u32>,
    pub total_duration: u64,
    pub load_duration: u64,
    pub prompt_eval_count: u64,
    pub prompt_eval_duration: u64,
    pub eval_count: u64,
    pub eval_duration: u64,
}

#[derive(Serialize)]
pub struct OllamaChat {
    pub model: &'static str,
    pub prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

const LL_MODEL: &str = "llama3.2";

fn get_prompt_to_extract_entities(text: String, labels: &Vec<String>) -> String {
    format!(
        r#"
    You are a data analyst who is helping me extract named entities from my data.
    Reply in CSV format only with these headings:

    EntityType,MatchedText

    Use these as possible EntityType:
    {}

    Exctract EntityType and MatchingText from the following:

    -------------------------------------------------

    {}
    "#,
        labels.join("\n"),
        text
    )
}

pub fn extract_entities(
    text: String,
    labels: &Vec<String>,
    ollama_host: String,
    ollama_port: u16,
) -> PiResult<Vec<ExtractedEntity>> {
    let payload = OllamaChat {
        model: LL_MODEL,
        prompt: get_prompt_to_extract_entities(text, labels),
        stream: Some(false),
    };

    let client = Client::new();
    let response = client
        .post(format!(
            "http://{}:{}/api/generate",
            ollama_host, ollama_port
        ))
        .header("content-type", "application/json")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .unwrap();

    let response = response.json::<OllamaResponse>()?;
    Ok(extract_entites_from_lines(response.response.as_str()))
}

fn get_prompt_to_classify(text: &String, labels: &Vec<String>) -> String {
    format!(
        r#"
You have these labels: {}
Please reply with only one label to classify the following text, add no other text:

{}
    "#,
        labels.join("\n"),
        text
    )
}

pub fn classify(
    text: &String,
    labels: &Vec<String>,
    ollama_host: String,
    ollama_port: u16,
) -> PiResult<String> {
    let payload = OllamaChat {
        model: LL_MODEL,
        prompt: get_prompt_to_classify(text, labels),
        stream: Some(false),
    };

    let client = Client::new();
    let response = client
        .post(format!(
            "http://{}:{}/api/generate",
            ollama_host, ollama_port
        ))
        .header("content-type", "application/json")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .unwrap();

    let response = response.json::<OllamaResponse>()?;
    let classification = response.response.clone();
    if classification.is_empty() {
        return Err(PiError::CouldNotClassifyText);
    }
    if !labels.contains(&classification) {
        return Err(PiError::CouldNotClassifyText);
    }
    Ok(classification)
}
