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
use log::debug;
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

// #[derive(Serialize)]
// pub struct ClaudeBatchRequest {
//     pub requests: Vec<ClaudeBatchItem>,
// }

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
}

#[derive(Serialize)]
pub struct ClaudeChatMessage {
    pub role: &'static str,
    pub content: String,
}

#[derive(Serialize)]
pub struct  SearchTermPromptInputContent {
    pub content_type: String,
    pub content: String,
}

#[derive(Deserialize)]
pub struct  SearchTermPromptOutputItem {
    pub search_term: String,
    pub relevance: String,
}

#[derive(Serialize)]
pub struct SearchTermPromptInput {
    pub topic: String,
    pub content: Vec<SearchTermPromptInputContent>,
}

const LL_MODEL: &str = "claude-3-5-haiku-20241022";

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
    api_key: &str,
) -> PiResult<Vec<ExtractedEntity>> {
    let payload = ClaudeChat {
        model: LL_MODEL,
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
        model: LL_MODEL,
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
You are a seasoned data professional. You will help me extract search terms from content blocks, based on a topic.
I will provide this data to you in JSON format.

Guidelines for search term extraction:
1. Extract search terms directly from content that match the topic
2. Identify contextually relevant terms based on topic-content relationship
3. Include relevant synonyms and variations
4. Consider multi-word combinations when appropriate
5. Add semantically related terms even if not present in content
6. Rate each term's relevance (broad/regular/tight) to the topic
7. Focus on precision and contextual accuracy
8. Ensure terms are topically coherent and meaningful

Please provide your response in JSON array format where each object contains exactly two fields:
1. "search_term" - containing the extracted search term
2. "relevance" - containing one of these values: "broad", "regular", or "tight"

Do not include any explanations, code fences or additional text.
"#,
    );
    let user_prompt = format!(
        r#"
{}
"#,
        serde_json::to_string(input).unwrap()
        // topic,
        // content.iter().map(|content_item| format!("<content>\n{}\n</content>", content_item)).collect::<Vec<String>>().join("\n"),
    );
    (system_prompt, user_prompt)
}

pub fn extract_search_terms(
    topic: String,
    content: &Vec<(String, String)>,
    api_key: &str,
) -> PiResult<Vec<SearchTermPromptOutputItem>> {
    let (system_prompt, user_prompt) = get_prompt_to_extract_search_terms(&SearchTermPromptInput{
        topic,
        content: content.iter().map(|content_item| SearchTermPromptInputContent {
            content_type: content_item.0.to_string(),
            content: content_item.1.to_string(),
        }).collect()
    });
    let payload = ClaudeChat {
        model: LL_MODEL,
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
            debug!("Failed to parse response for Search Term Extraction from Claude as JSON: {}", e);
            return Err(PiError::AnthropicServiceError(
                "Failed to parse response for Search Term Extraction from Claude as JSON".to_string()
            ));
        }
    };
    Ok(match serde_json::from_str::<Vec<serde_json::Value>>(response.content[0].text.as_str()) {
        Ok(prompt_response_json_object) => {
            prompt_response_json_object.iter().map(|json_object| {
                match json_object.get("search_term") {
                    Some(search_term) => {
                        match json_object.get("relevance") {
                            Some(relevance) => Some(SearchTermPromptOutputItem {
                                search_term: search_term.as_str().unwrap().to_string(),
                                relevance: relevance.as_str().unwrap().to_string(),
                            }),
                            None => {
                                debug!(
                                    "Failed to parse search term item's relevance from Claude response: {:?}",
                                    json_object
                                );
                                None
                            }
                        }
                    },
                    None => {
                        debug!(
                            "Failed to parse search term item from Claude response: {:?}",
                            json_object
                        );
                        None
                    }
                }
            }).filter_map(|x| x).collect::<Vec<SearchTermPromptOutputItem>>()
        },
        Err(_) => {
            debug!(
                "Failed to parse response for Search Term Extraction from Claude as JSON: {}",
                response.content[0].text
            );
            return Ok(vec![]);
        }
    })
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
