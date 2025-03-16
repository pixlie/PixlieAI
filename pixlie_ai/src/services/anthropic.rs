// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::services::llm::{LLMResponse, LLM};
use crate::workspace::{APIProvider, WorkspaceCollection};
use crate::{
    entity::ExtractedEntity,
    error::{PiError, PiResult},
    services::extract_entites_from_lines,
    FetchRequest,
};
use log::debug;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, CONTENT_TYPE};
use reqwest::Method;
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

// #[derive(Serialize)]
// pub struct ClaudeBatchRequest {
//     pub requests: Vec<ClaudeBatchItem>,
// }

// #[derive(Serialize)]
// pub struct ClaudeBatchItem {
//     pub custom_id: String,
//     pub params: ClaudeChat,
// }

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

impl LLM for Anthropic {
    fn get_prompt_for_objective(pixlie_schema: &String, objective: &String) -> PiResult<String> {
        Ok(format!(
            r#"
        I have a software that you can interact with using this schema:
        {}

        I have the following objective:
        {}

        Please respond in JSON with LLMResponse:
        "#,
            pixlie_schema, objective
        ))
    }

    fn get_request(
        pixlie_schema: &String,
        objective: &String,
        calling_node_id: u32,
    ) -> PiResult<FetchRequest> {
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
                content: Self::get_prompt_for_objective(pixlie_schema, objective)?,
            }],
        };
        let serialized_payload = serde_json::to_string(&payload)?;
        request.body = Some(serialized_payload);
        Ok(request)
    }

    fn parse_response(response: &str) -> PiResult<Vec<LLMResponse>> {
        let claude_response: ClaudeResponse = serde_json::from_str(response)?;
        Ok(claude_response
            .content
            .into_iter()
            .map(|claude_response_content| LLMResponse {
                content_type: claude_response_content._type,
                content: claude_response_content.text,
            })
            .collect())
    }
}

fn get_prompt_to_extract_entities(text: String, labels: &Vec<String>) -> String {
    format!(
        r#"
    You are a data analyst who is helping me extract named entities from my data.
    Reply in CSV format only with these headings:

    EntityType,MatchedText

    Use these as possible EntityType:
    {}

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
    api_key: &str,
) -> PiResult<Vec<ExtractedEntity>> {
    let payload = ClaudeChat {
        model: LL_MODEL_HAIKU,
        max_tokens: 1024,
        temperature: None,
        system: None,
        messages: vec![ClaudeChatMessage {
            role: "user",
            content: get_prompt_to_extract_entities(text, labels),
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

fn get_prompt_to_classify(text: &String, labels: &Vec<String>) -> String {
    format!(
        r#"
    You are a data analyst who is helping me classify the following text.
    Please classify the text as one one of the following labels:

    {}

    Classify the following text and reply only with the label:
    -------------------------------------------------

    {}
    "#,
        labels.join("\n"),
        text
    )
}

pub fn classify(text: &String, labels: &Vec<String>, api_key: &str) -> PiResult<String> {
    let payload = ClaudeChat {
        model: LL_MODEL_HAIKU,
        max_tokens: 1024,
        temperature: None,
        system: None,
        messages: vec![ClaudeChatMessage {
            role: "user",
            content: get_prompt_to_classify(&text, labels),
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
    let classification = response.content[0].text.clone();
    if classification.is_empty() {
        return Err(PiError::CouldNotClassifyText);
    }
    if !labels.contains(&classification) {
        return Err(PiError::CouldNotClassifyText);
    }
    Ok(classification)
}

pub fn get_prompt_to_extract_search_terms(input: &SearchTermPromptInput) -> (String, String) {
    let system_prompt = format!(
        r#"
You are a data professional and a linguistic expert specializing in semantic analysis.
Your task is to extract search terms from content blocks based on a specific topic.
I will provide structured input in JSON format with a topic and content blocks.

Guidelines for search term extraction:
1. Extract only exact phrases from the content that are 1-3 consecutive words long.
2. Only include terms that are directly relevant to the specified topic.
3. Each search term must appear verbatim in the source content.
4. Consider the content_type field when determining a term's importance (e.g., titles and headings should be prioritized).
5. Focus on high precision and contextual accuracy.

Output format:
Return a JSON array of strings with exactly this structure:
[\"exact phrase one\", \"exact phrase two\", ...]

Do not include any explanations, code fences, or text outside the JSON array.
"#,
    );
    let user_prompt = format!(
        r#"
{}
"#,
        serde_json::to_string(input).unwrap()
    );
    (system_prompt, user_prompt)
}

pub fn extract_search_terms(
    topic: String,
    content: &Vec<(String, String)>,
    api_key: &str,
) -> PiResult<Vec<SearchTermPromptOutputItem>> {
    let (system_prompt, user_prompt) = get_prompt_to_extract_search_terms(&SearchTermPromptInput {
        topic,
        content: content
            .iter()
            .map(|content_item| SearchTermPromptInputContent {
                content_type: content_item.0.to_string(),
                content: content_item.1.to_string(),
            })
            .collect(),
    });
    let payload = ClaudeChat {
        model: LL_MODEL_HAIKU,
        max_tokens: 1024,
        system: Some(system_prompt),
        temperature: Some(0.1),
        messages: vec![ClaudeChatMessage {
            role: "user",
            content: user_prompt,
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

    let response = match response.json::<ClaudeResponse>() {
        Ok(response) => response,
        Err(e) => {
            debug!(
                "Failed to parse response for Search Term Extraction from Claude as JSON: {}",
                e
            );
            return Err(PiError::AnthropicServiceError(
                "Failed to parse response for Search Term Extraction from Claude as JSON"
                    .to_string(),
            ));
        }
    };
    Ok(
        match serde_json::from_str::<Vec<serde_json::Value>>(response.content[0].text.as_str()) {
            Ok(prompt_response_json_object) => prompt_response_json_object
                .iter()
                .filter_map(|search_term| match search_term.as_str() {
                    Some(search_term) => Some(SearchTermPromptOutputItem(search_term.to_string())),
                    None => {
                        debug!(
                            "Failed to parse search term from Claude response: {}",
                            search_term
                        );
                        None
                    }
                })
                .collect::<Vec<SearchTermPromptOutputItem>>(),
            Err(_) => {
                debug!(
                    "Failed to parse response for Search Term Extraction from Claude as JSON: {}",
                    response.content[0].text
                );
                return Ok(vec![]);
            }
        },
    )
}

// pub fn extract_entities_in_batch(
//     extraction_request: Vec<ExtractionRequest>,
//     api_key: &str,
// ) -> PiResult<Vec<ExtractedEntity>> {
//     let payload = ClaudeBatchRequest {
//         requests: extraction_request
//             .iter()
//             .map(|x| ClaudeBatchItem {
//                 custom_id: "".to_string(),
//                 params: ClaudeChat {
//                     model: "claude-3-haiku-20240307",
//                     max_tokens: 1024,
//                     messages: vec![ClaudeChatMessage {
//                         role: "user",
//                         content: get_prompt(x),
//                     }],
//                 },
//             })
//             .collect(),
//     };

//     let client = Client::new();
//     let response = client
//         .post("https://api.anthropic.com/v1/messages")
//         .header("x-api-key", api_key)
//         .header("anthropic-version", "2023-06-01")
//         .header("anthropic-beta", "message-batches-2024-09-24")
//         .header("content-type", "application/json")
//         .body(serde_json::to_string(&payload).unwrap())
//         .send()
//         .unwrap();

//     let response = response.json::<ClaudeResponse>()?;
//     Ok(extract_entites_from_lines(
//         response.content[0].text.as_str(),
//     ))
// }
