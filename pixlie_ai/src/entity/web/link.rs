use crate::engine::{CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeId, Payload};
use crate::entity::web::domain::Domain;
use crate::entity::web::web_page::WebPage;
use crate::error::PiResult;
use crate::utils::fetcher::FetchEvent;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use url::Url;

// A link that should fetch
#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, TS)]
pub struct Link {
    pub url: String,
    pub is_fetched: bool,
}

impl Link {
    pub fn add(url: &String, engine: &Engine) -> PiResult<()> {
        match Url::parse(url) {
            Ok(parsed) => match parsed.domain() {
                Some(domain) => {
                    let link_node_id = engine.add_node(
                        Payload::Link(Link {
                            url: url.to_string(),
                            is_fetched: false,
                        }),
                        vec![CommonNodeLabels::AddedByUser.to_string()],
                    );
                    let domain_node_id = engine.add_node(
                        Payload::Domain(Domain {
                            name: domain.to_string(),
                            is_allowed_to_crawl: true,
                            last_fetched_at: None,
                        }),
                        vec![CommonNodeLabels::AddedByUser.to_string()],
                    );
                    engine.add_connection(
                        (link_node_id, domain_node_id),
                        (
                            CommonEdgeLabels::Related.to_string(),
                            CommonEdgeLabels::Related.to_string(),
                        ),
                    );
                }
                None => {
                    error!("Can not parse URL to get domain: {}", &url);
                }
            },
            Err(err) => match err {
                _ => {
                    error!("Can not parse URL to get domain: {}", &url);
                }
            },
        };
        Ok(())
    }
}

impl Node for Link {
    fn get_label() -> String {
        "Link".to_string()
    }

    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<Link> {
        debug!("Processing Link node: {}", self.url);
        // Download the linked URL and add a new WebPage node
        if self.is_fetched {
            return None;
        }

        match engine.fetch_url(self, node_id) {
            Ok(rx) => match rx.recv() {
                Ok(event) => match event {
                    FetchEvent::FetchResponse(id, url, contents) => {
                        debug!("Fetched HTML from {}", self.url);
                        let content_node_id = engine.add_node(
                            Payload::FileHTML(WebPage {
                                contents,
                                ..Default::default()
                            }),
                            vec![],
                        );
                        engine.add_connection(
                            (node_id.clone(), content_node_id),
                            (
                                CommonEdgeLabels::Content.to_string(),
                                CommonEdgeLabels::Path.to_string(),
                            ),
                        );
                        Some(Link {
                            is_fetched: true,
                            ..self.clone()
                        })
                    }
                    _ => None,
                },
                Err(_err) => None,
            },
            Err(_err) => None,
        }
    }
}
