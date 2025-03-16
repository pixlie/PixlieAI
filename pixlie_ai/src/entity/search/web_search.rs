use crate::engine::node::{NodeItem, Payload};
use crate::engine::Engine;
use crate::error::{PiError, PiResult};
use crate::workspace::{APIProvider, WorkspaceCollection};
use crate::{ExternalData, FetchRequest};
use reqwest::header::{HeaderMap, ACCEPT};
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
