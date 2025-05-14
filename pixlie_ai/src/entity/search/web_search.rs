// TODO: Remove the following when we start using this module
#![allow(dead_code)]

use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
use crate::entity::project_settings::ProjectSettings;
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::workspace::{APIProvider, WorkspaceCollection};
use crate::{ExternalData, FetchRequest};
use reqwest::header::{HeaderMap, ACCEPT};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use url::Url;

pub struct WebSearch;

impl WebSearch {
    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        // We send a web search request using Brave Search API
        match data_from_previous_request {
            Some(data) => {
                // When we get a response from Brave Search, we create Link nodes for each URL
                match data {
                    ExternalData::Response(response) => {
                        let response: BraveSearchResponse =
                            match serde_json::from_str(&response.contents) {
                                Ok(response) => response,
                                Err(err) => {
                                    return Err(PiError::InternalError(format!(
                                        "Failed to parse Brave Search response: {}",
                                        err
                                    )));
                                }
                            };
                        // Skip if project settings are not available
                        let project_settings = match engine
                            .get_node_ids_with_label(&NodeLabel::ProjectSettings)
                        {
                            settings_node_ids => {
                                if settings_node_ids.is_empty() {
                                    return Err(PiError::GraphError(
                                        "No ProjectSettings node found".to_string(),
                                    ));
                                }
                                match engine.get_node_by_id(&settings_node_ids[0]) {
                                    Some(settings_node) => {
                                        if settings_node
                                            .labels
                                            .contains(&NodeLabel::ProjectSettings)
                                        {
                                            match &settings_node.payload {
                                                Payload::ProjectSettings(settings) => {
                                                    settings.clone()
                                                }
                                                _ => {
                                                    return Err(PiError::GraphError(
                                                        "No ProjectSettings node found".to_string(),
                                                    ));
                                                }
                                            }
                                        } else {
                                            return Err(PiError::GraphError(
                                                "No ProjectSettings node found".to_string(),
                                            ));
                                        }
                                    }
                                    None => {
                                        return Err(PiError::GraphError(
                                            "No ProjectSettings node found".to_string(),
                                        ));
                                    }
                                }
                            }
                        };
                        for result in response.web.results {
                            let url = Url::parse(&result.url);
                            if url.is_err() {
                                continue;
                            }
                            let url = url.unwrap();
                            let domain = match url.domain() {
                                Some(domain) => domain.to_string(),
                                None => continue,
                            };
                            if project_settings.is_domain_allowed(
                                &node.id,
                                domain,
                                engine.clone(),
                            )? {
                                let link_node_id = Link::add(
                                    engine.clone(),
                                    &result.url,
                                    vec![NodeLabel::AddedByWebSearch, NodeLabel::Link],
                                    vec![],
                                    true,
                                )?;
                                engine.add_connection(
                                    (node.id.clone(), link_node_id.clone()),
                                    (EdgeLabel::Suggests, EdgeLabel::SuggestedFor),
                                )?;
                            }
                        }
                        engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                    }
                    _ => {}
                }
            }
            None => {
                // Skip if there is no Brave Search API key
                let api_key = match WorkspaceCollection::get_default()?
                    .get_api_key(&APIProvider::BraveSearch)
                {
                    Some(api_key) => api_key.to_string(),
                    None => return Err(PiError::ApiKeyNotConfigured("Brave Search".to_string())),
                };
                // Skip if project settings are not available
                let project_settings =
                    match engine.get_node_ids_with_label(&NodeLabel::ProjectSettings) {
                        settings_node_ids => {
                            if settings_node_ids.is_empty() {
                                return Err(PiError::GraphError(
                                    "No ProjectSettings node found".to_string(),
                                ));
                            }
                            match engine.get_node_by_id(&settings_node_ids[0]) {
                                Some(settings_node) => {
                                    if settings_node.labels.contains(&NodeLabel::ProjectSettings) {
                                        match &settings_node.payload {
                                            Payload::ProjectSettings(settings) => settings.clone(),
                                            _ => {
                                                return Err(PiError::GraphError(
                                                    "No ProjectSettings node found".to_string(),
                                                ));
                                            }
                                        }
                                    } else {
                                        return Err(PiError::GraphError(
                                            "No ProjectSettings node found".to_string(),
                                        ));
                                    }
                                }
                                None => {
                                    return Err(PiError::GraphError(
                                        "No ProjectSettings node found".to_string(),
                                    ));
                                }
                            }
                        }
                    };
                let mut url = Url::parse("https://api.search.brave.com/res/v1/web/search")?;
                match &node.payload {
                    Payload::Text(search_term) => {
                        url.query_pairs_mut().append_pair("q", search_term);

                        let mut request = FetchRequest::new(node.id, url.as_str());
                        request.headers = HeaderMap::from_iter(vec![
                            (ACCEPT, "application/json".parse().unwrap()),
                            (
                                "X-Subscription-Token".parse().unwrap(),
                                api_key.parse().unwrap(),
                            ),
                        ]);
                        engine.fetch_api(request)?;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct BraveSearchResponse {
    // pub query: Query,
    // pub mixed: Mixed,
    // #[serde(rename = "type")]
    // pub type_field: String,
    // pub videos: Videos,
    pub web: Web,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Query {
    pub original: String,
    pub show_strict_warning: bool,
    pub is_navigational: bool,
    pub is_news_breaking: bool,
    pub spellcheck_off: bool,
    pub country: String,
    pub bad_results: bool,
    pub should_fallback: bool,
    pub postal_code: String,
    pub city: String,
    pub header_country: String,
    pub more_results_available: bool,
    pub state: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Mixed {
    #[serde(rename = "type")]
    pub type_field: String,
    pub main: Vec<Main>,
    pub top: Vec<Value>,
    pub side: Vec<Value>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Main {
    #[serde(rename = "type")]
    pub type_field: String,
    pub index: Option<i64>,
    pub all: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Videos {
    #[serde(rename = "type")]
    pub type_field: String,
    pub results: Vec<VideoResult>,
    pub mutated_by_goggles: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct VideoResult {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub age: Option<String>,
    pub page_age: Option<String>,
    pub video: Video,
    pub meta_url: MetaUrl,
    pub thumbnail: Thumbnail,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Video {
    pub duration: Option<String>,
    pub views: Option<i64>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct MetaUrl {
    pub scheme: String,
    pub netloc: String,
    pub hostname: String,
    pub favicon: String,
    pub path: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Thumbnail {
    pub src: String,
    pub original: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Web {
    #[serde(rename = "type")]
    pub type_field: String,
    pub results: Vec<WebResult>,
    pub family_friendly: bool,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct WebResult {
    pub title: String,
    pub url: String,
    pub is_source_local: bool,
    pub is_source_both: bool,
    pub description: Option<String>,
    pub page_age: Option<String>,
    pub profile: Option<Profile>,
    pub language: Option<String>,
    pub family_friendly: bool,
    #[serde(rename = "type")]
    pub type_field: String,
    pub subtype: String,
    pub is_live: bool,
    pub meta_url: Option<MetaUrl2>,
    pub thumbnail: Option<Thumbnail2>,
    pub age: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Profile {
    pub name: String,
    pub url: String,
    pub long_name: String,
    pub img: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct MetaUrl2 {
    pub scheme: String,
    pub netloc: String,
    pub hostname: String,
    pub favicon: String,
    pub path: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
pub struct Thumbnail2 {
    pub src: String,
    pub original: String,
    pub logo: bool,
}
