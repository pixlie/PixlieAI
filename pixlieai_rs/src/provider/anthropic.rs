// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use super::{EntityExtraction, EntityExtractionProvider};
use crate::{
    entity::{EntityType, ExtractedEntity},
    error::PiResult,
    provider::extract_entites_from_lines,
    GraphEntity,
};
use log::{error, info};
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
pub struct ClaudeChatModel {
    pub model: &'static str,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeChatMessage>,
}

#[derive(Serialize)]
pub struct ClaudeChatMessage {
    pub role: &'static str,
    pub content: String,
}

pub async fn extract_entities<T>(payload: &T, api_key: &str) -> PiResult<Vec<ExtractedEntity>>
where
    T: EntityExtraction,
{
    let mut extracted: Vec<ExtractedEntity> = vec![];

    let prompt = format!(
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
        payload.get_labels().join("\n"),
        payload.get_payload()
    );

    info!("Prompt: {}", &prompt);
    let payload = ClaudeChatModel {
        model: "claude-3-haiku-20240307",
        max_tokens: 1024,
        messages: vec![ClaudeChatMessage {
            role: "user",
            content: prompt,
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .await
        .unwrap();

    let response = response.json::<ClaudeResponse>().await?;
    Ok(extract_entites_from_lines(
        response.content[0].text.as_str(),
    ))
}
