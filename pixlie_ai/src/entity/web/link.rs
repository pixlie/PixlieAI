use crate::engine::{CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeId, Payload};
use crate::entity::web::domain::Domain;
use crate::entity::web::web_page::WebPage;
use crate::error::PiResult;
use crate::utils::fetcher::FetchEvent;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::thread;
use ts_rs::TS;
use url::Url;

// A link that should fetch
#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, TS)]
pub struct Link {
    pub url: String,
    pub is_fetched: bool,
}

impl Link {
    pub fn add(engine: &Engine, url: &String) -> PiResult<()> {
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
                    error!("Can not parse URL {} to get domain", &url);
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

    fn process(&self, engine: Arc<&Engine>, node_id: &NodeId) {
        // Download the linked URL and add a new WebPage node
        if self.is_fetched {
            return;
        }
        debug!("Processing Link node: {}", self.url);

        let node_id = node_id.clone();
        let url = self.url.clone();
        let link = self.clone();
        let engine = engine.clone();

        thread::scope(|scope| {
            scope.spawn(|| {
                match engine.fetch_url(url.clone(), &node_id) {
                    Ok(rx) => match rx.recv() {
                        Ok(event) => match event {
                            FetchEvent::FetchResponse(_id, _url, contents) => {
                                debug!("Fetched HTML from {}", url);
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
                                engine.update_node(
                                    &node_id,
                                    Payload::Link(Link {
                                        is_fetched: true,
                                        ..link
                                    }),
                                );
                            }
                            _ => {}
                        },
                        Err(_err) => {
                            error!("Can not fetch HTML from {}", url);
                        }
                    },
                    Err(_err) => {
                        error!("Can not fetch HTML from {}", url);
                    }
                }
            });
        });

        // Returning to the engine, the crawl will continue in the separate thread
        debug!("Returning from Link node: {}", self.url);
    }
}
