use crate::engine::node::{NodeItem, NodeLabel, Payload};
use crate::engine::{CommonEdgeLabels, Engine, NodeFlags};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use crate::workspace::{APIProvider, WorkspaceCollection};
use crate::{ExternalData, FetchRequest};
use reqwest::header::{HeaderMap, ACCEPT};
use serde::{Deserialize, Serialize};
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
                                    return Err(PiError::InternalError(
                                        "Failed to parse Brave Search response".to_string(),
                                    ));
                                }
                            };
                        for result in response.web.results {
                            let link_node_id = Link::add(
                                engine.clone(),
                                &result.url,
                                vec![NodeLabel::AddedByWebSearch],
                                vec![],
                                true,
                            )?;
                            engine.add_connection(
                                (node.id.clone(), link_node_id.clone()),
                                (
                                    CommonEdgeLabels::Suggests.to_string(),
                                    CommonEdgeLabels::SuggestedFor.to_string(),
                                ),
                            )?;
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
                let mut url = Url::parse("https://api.search.brave.com/res/v1/web/search")?;
                let search_term = match &node.payload {
                    Payload::Text(search_term) => search_term,
                    _ => {
                        return Err(PiError::GraphError(format!(
                            "Expected Payload::Text, got {}",
                            node.payload.to_string()
                        )));
                    }
                };
                url.query_pairs_mut().append_pair("q", search_term);

                let mut request = FetchRequest::new(node.id, url.as_str());
                request.headers = HeaderMap::from_iter(vec![
                    (ACCEPT, "application/json".parse().unwrap()),
                    (
                        "X-Subscription-Token".parse().unwrap(),
                        api_key.parse().unwrap(),
                    ),
                ]);
                engine.fetch_api(request)?
            }
        }
        Ok(())
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BraveSearchResponse {
    pub query: Query,
    pub mixed: Mixed,
    #[serde(rename = "type")]
    pub type_field: String,
    pub videos: Videos,
    pub web: Web,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Query {
    pub original: String,
    #[serde(rename = "show_strict_warning")]
    pub show_strict_warning: bool,
    #[serde(rename = "is_navigational")]
    pub is_navigational: bool,
    #[serde(rename = "is_news_breaking")]
    pub is_news_breaking: bool,
    #[serde(rename = "spellcheck_off")]
    pub spellcheck_off: bool,
    pub country: String,
    #[serde(rename = "bad_results")]
    pub bad_results: bool,
    #[serde(rename = "should_fallback")]
    pub should_fallback: bool,
    #[serde(rename = "postal_code")]
    pub postal_code: String,
    pub city: String,
    #[serde(rename = "header_country")]
    pub header_country: String,
    #[serde(rename = "more_results_available")]
    pub more_results_available: bool,
    pub state: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Mixed {
    #[serde(rename = "type")]
    pub type_field: String,
    pub main: Vec<Main>,
    pub top: Vec<Value>,
    pub side: Vec<Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Main {
    #[serde(rename = "type")]
    pub type_field: String,
    pub index: Option<i64>,
    pub all: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Videos {
    #[serde(rename = "type")]
    pub type_field: String,
    pub results: Vec<VideoResult>,
    #[serde(rename = "mutated_by_goggles")]
    pub mutated_by_goggles: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoResult {
    #[serde(rename = "type")]
    pub type_field: String,
    pub url: String,
    pub title: String,
    pub description: String,
    pub age: Option<String>,
    #[serde(rename = "page_age")]
    pub page_age: Option<String>,
    pub video: Video,
    #[serde(rename = "meta_url")]
    pub meta_url: MetaUrl,
    pub thumbnail: Thumbnail,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Video {
    pub duration: Option<String>,
    pub views: Option<i64>,
    pub creator: Option<String>,
    pub publisher: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaUrl {
    pub scheme: String,
    pub netloc: String,
    pub hostname: String,
    pub favicon: String,
    pub path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail {
    pub src: String,
    pub original: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Web {
    #[serde(rename = "type")]
    pub type_field: String,
    pub results: Vec<WebResult>,
    #[serde(rename = "family_friendly")]
    pub family_friendly: bool,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebResult {
    pub title: String,
    pub url: String,
    #[serde(rename = "is_source_local")]
    pub is_source_local: bool,
    #[serde(rename = "is_source_both")]
    pub is_source_both: bool,
    pub description: String,
    #[serde(rename = "page_age")]
    pub page_age: String,
    pub profile: Profile,
    pub language: String,
    #[serde(rename = "family_friendly")]
    pub family_friendly: bool,
    #[serde(rename = "type")]
    pub type_field: String,
    pub subtype: String,
    #[serde(rename = "is_live")]
    pub is_live: bool,
    #[serde(rename = "meta_url")]
    pub meta_url: MetaUrl2,
    pub thumbnail: Option<Thumbnail2>,
    pub age: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    pub name: String,
    pub url: String,
    #[serde(rename = "long_name")]
    pub long_name: String,
    pub img: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetaUrl2 {
    pub scheme: String,
    pub netloc: String,
    pub hostname: String,
    pub favicon: String,
    pub path: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Thumbnail2 {
    pub src: String,
    pub original: String,
    pub logo: bool,
}
