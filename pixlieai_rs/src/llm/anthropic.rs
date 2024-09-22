// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use super::{EntityExtractionWithLLM, LLMProvider, LargeLanguageModel};
use crate::{
    entity::{EntityType, ExtractedEntity},
    GraphEntity,
};
use log::{error, info};
use serde::{Deserialize, Serialize};

impl LargeLanguageModel {
    pub fn get_models_for_anthropic() -> Vec<LargeLanguageModel> {
        vec![
            LargeLanguageModel {
                label: "Claude 3 Haiku".to_string(),
                ai_provider: LLMProvider::Anthropic,
                api_name: "claude-3-haiku-20240307".to_string(),
                context_window: Some(200_000),
                price_per_million_input_tokens: Some(25),
                price_per_million_output_tokens: Some(125),
            },
            LargeLanguageModel {
                label: "Claude 3.5 Sonnet".to_string(),
                ai_provider: LLMProvider::Anthropic,
                api_name: "claude-3-5-sonnet-20240620".to_string(),
                context_window: Some(200_000),
                price_per_million_input_tokens: Some(300),
                price_per_million_output_tokens: Some(1500),
            },
        ]
    }
}

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
    pub model: String,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeChatMessage>,
}

#[derive(Serialize)]
pub struct ClaudeChatMessage {
    pub role: String,
    pub content: String,
}

pub async fn extract_entities_from_email_with_llm<T>(payload: &T) -> Vec<ExtractedEntity>
where
    T: EntityExtractionWithLLM,
{
    let payload_content_type = payload.get_payload_content_type();
    let entity_types = payload.get_extractable_entity_types();
    let example_text = payload.get_example_text();
    let example_extractions = payload.get_example_extractions();
    let mut entities: Vec<ExtractedEntity> = vec![];

    let prompt: String = format!(
        r#"
Analyze the following {} and extract relevant entities.
Present the extracted information in CSV format with the following structure:

Entity_Type,Matched_Text

Use these Entity_Types:
{}

Only list rows that have clear Entity_Type. Double quote the Matched_Text.
Do not add prelude to the output.

Here is an example {}:
{}

Here are the extracted entities:
{}
"#,
        payload_content_type,
        entity_types
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<String>>()
            .join("\n"),
        payload_content_type,
        example_text,
        example_extractions
            .iter()
            .map(|x| format!("{},{}", x.0.to_string(), x.1.to_string()))
            .collect::<Vec<String>>()
            .join("\n"),
    );

    // info!("Prompt: {}", prompt);
    let payload = ClaudeChatModel {
        model: "claude-3-haiku-20240307".to_string(),
        max_tokens: 1024,
        messages: vec![ClaudeChatMessage {
            role: "user".to_string(),
            content: prompt,
        }],
    };

    let client = reqwest::Client::new();
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", "")
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .body(serde_json::to_string(&payload).unwrap())
        .send()
        .await
        .unwrap();

    match response.json::<ClaudeResponse>().await {
        Ok(response) => {
            let text = response.content[0].text.as_str();
            let mut reader = csv::Reader::from_reader(text.as_bytes());
            for line in reader.records() {
                match line {
                    Ok(line) => {
                        entities.push(ExtractedEntity {
                            entity_type: EntityType::try_from(
                                line.get(0).unwrap().to_string().as_str(),
                            )
                            .unwrap(),
                            matching_text: line.get(1).unwrap().to_string(),
                        });
                    }
                    Err(_) => {}
                }
            }
        }
        Err(err) => {
            error!("Error parsing response from Claude: {}", err);
        }
    }

    // Log the entities
    info!(
        "Extracted entities:\n{}",
        entities
            .iter()
            .map(|x| format!("{},{}", x.entity_type.to_string(), x.matching_text.as_str()))
            .collect::<Vec<String>>()
            .join("\n")
    );
    entities
}
