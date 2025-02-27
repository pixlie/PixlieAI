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

pub fn get_prompt_to_extract_search_terms(topic: String, content: &Vec<String>) -> (String, String) {

    let system_prompt = format!(
        r#"
You are a seasoned data professional who is helping me extract search terms based on topics of interest I provide to you.
I will use these search terms to look for matching content that is relevant to a topic of interest.

I will provide you with content blocks that contain relevant content.

How to determine search terms:
- You will look for relevant search terms in the content that match a topic.
- You will also add synonyms for these search terms to your results.
- You may provide multiple combinations of words too.
- You may provide search terms that do not appear in the content too, based on your understanding of what I am looking for.

Reply in CSV format ONLY with these headings:
Topic,SearchTerm,Match

There should be one row for each combination of Topic & SearchTerm.
The value of Match should be 'broad','regular' or 'tight', based on how relevant the search term is for the topic.
"#,
    );
    let user_prompt = format!(
        r#"
Topic: `{}`

Content:
{}
"#,
        topic,
        content.iter().map(|content_item| format!("<content>\n{}\n</content>", content_item)).collect::<Vec<String>>().join("\n"),
    );
    (system_prompt, user_prompt)
}

pub fn extract_search_terms(
    topic: String,
    content: &Vec<String>,
    api_key: &str,
) -> PiResult<Vec<(String, String, String)>> {
    let (system_prompt, user_prompt) = get_prompt_to_extract_search_terms(topic, content);
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

    let response = response.json::<ClaudeResponse>()?;
    let lines = response.content[0].text.lines().collect::<Vec<&str>>();
    let mut search_terms = vec![];
    for line in lines {
        let parts = line.split(",").collect::<Vec<&str>>();
        if parts.len() != 3 {
            continue;
        }
        search_terms.push((parts[0].trim().to_string(), parts[1].trim().to_string(), parts[2].trim().to_string()));
    }
    Ok(search_terms)
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
