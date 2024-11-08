// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use super::ExtractionRequest;
use crate::{
    entity::ExtractedEntity, error::PiResult, services::extract_entites_from_lines, GraphEntity,
};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct ClaudeResponse {
    pub id: String,
    pub content: Vec<ClaudeResponseContent>,
}

#[derive(Debug, Deserialize)]
pub struct ClaudeResponseContent {
    #[serde(rename = "type")]
    pub _type: String,
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct ClaudeContentText {
    pub entities: Vec<GraphEntity>,
}

#[derive(Serialize)]
pub struct ClaudeBatchRequest {
    pub requests: Vec<ClaudeBatchItem>,
}

#[derive(Serialize)]
pub struct ClaudeBatchItem {
    pub custom_id: String,
    pub params: ClaudeChat,
}

#[derive(Serialize)]
pub struct ClaudeChat {
    pub model: &'static str,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeChatMessage>,
}

#[derive(Serialize)]
pub struct ClaudeChatMessage {
    pub role: &'static str,
    pub content: String,
}

pub fn get_prompt(extraction_request: &ExtractionRequest) -> String {
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
        extraction_request.labels.join("\n"),
        extraction_request.text
    )
}

pub fn extract_entities(
    extraction_request: ExtractionRequest,
    api_key: &str,
) -> PiResult<Vec<ExtractedEntity>> {
    let payload = ClaudeChat {
        model: "claude-3-haiku-20240307",
        max_tokens: 1024,
        messages: vec![ClaudeChatMessage {
            role: "user",
            content: get_prompt(&extraction_request),
        }],
    };

    let client = Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .unwrap();

    let response = response.json::<ClaudeResponse>()?;
    Ok(extract_entites_from_lines(
        response.content[0].text.as_str(),
    ))
}

pub fn extract_entities_in_batch(
    extraction_request: Vec<ExtractionRequest>,
    api_key: &str,
) -> PiResult<Vec<ExtractedEntity>> {
    let payload = ClaudeBatchRequest {
        requests: extraction_request
            .iter()
            .map(|x| ClaudeBatchItem {
                custom_id: "".to_string(),
                params: ClaudeChat {
                    model: "claude-3-haiku-20240307",
                    max_tokens: 1024,
                    messages: vec![ClaudeChatMessage {
                        role: "user",
                        content: get_prompt(x),
                    }],
                },
            })
            .collect(),
    };

    let client = Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("anthropic-beta", "message-batches-2024-09-24")
        .header("content-type", "application/json")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .unwrap();

    let response = response.json::<ClaudeResponse>()?;
    Ok(extract_entites_from_lines(
        response.content[0].text.as_str(),
    ))
}
