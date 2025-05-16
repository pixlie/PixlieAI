// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::utils::llm::LLMProvider;
use crate::workspace::{APIProvider, WorkspaceCollection};
use crate::{
    error::{PiError, PiResult},
    FetchRequest,
};
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use reqwest::Method;
use serde::de::DeserializeOwned;
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

#[derive(Serialize)]
pub struct ClaudeChat<'a> {
    pub model: &'a str,
    pub max_tokens: u32,
    pub messages: Vec<ClaudeChatMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Serialize)]
pub struct ClaudeChatMessage<'a> {
    pub role: &'a str,
    pub content: String,
}

#[derive(Serialize)]
pub struct SearchTermPromptInputContent {
    pub content_type: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct SearchTermPromptOutputItem(pub String);

#[derive(Serialize)]
pub struct SearchTermPromptInput {
    pub topic: String,
    pub content: Vec<SearchTermPromptInputContent>,
}

// const LL_MODEL_SONNET: &str = "claude-3-7-sonnet-latest";
const LL_MODEL_HAIKU: &str = "claude-3-5-haiku-latest";

pub struct Anthropic;
impl LLMProvider for Anthropic {
    fn get_request(prompt: &String, calling_node_id: u32) -> PiResult<FetchRequest> {
        // Skip if there is no Anthropic API key
        let key = match WorkspaceCollection::get_default()?.get_api_key(&APIProvider::Anthropic) {
            Some(key) => key.to_string(),
            None => return Err(PiError::ApiKeyNotConfigured("Anthropic".to_string())),
        };
        let mut request =
            FetchRequest::new(calling_node_id, "https://api.anthropic.com/v1/messages");
        request.method = Method::POST;
        request.headers = HeaderMap::from_iter(vec![
            (CONTENT_TYPE, "application/json".parse().unwrap()),
            (
                "anthropic-version".parse().unwrap(),
                "2023-06-01".parse().unwrap(),
            ),
            ("x-api-key".parse().unwrap(), key.parse().unwrap()),
        ]);
        let payload = ClaudeChat {
            model: LL_MODEL_HAIKU,
            max_tokens: 1024,
            temperature: None,
            system: None,
            messages: vec![ClaudeChatMessage {
                role: "user",
                content: prompt.to_string(),
            }],
        };
        let serialized_payload = serde_json::to_string(&payload)?;
        request.body = Some(serialized_payload);
        Ok(request)
    }

    fn parse_response<T: DeserializeOwned>(response: &str) -> PiResult<T> {
        let claude_response: ClaudeResponse = serde_json::from_str(response)?;
        if claude_response.content.len() != 1 {
            return Err(PiError::CouldNotParseResponseFromLLM(
                LL_MODEL_HAIKU.to_string(),
            ));
        }
        let text = &claude_response.content[0].text;
        // Check if the text starts a Markdown code block
        let payload = if text.starts_with("```json") {
            text.trim_start_matches("```json").trim_end_matches("```")
        } else if text.starts_with("```") {
            text.trim_start_matches("```").trim_end_matches("```")
        } else {
            text
        };
        serde_json::from_str(payload)
            .map_err(|e| PiError::CouldNotParseResponseFromLLM(format!("JSON parse error: {}", e)))
    }
}
