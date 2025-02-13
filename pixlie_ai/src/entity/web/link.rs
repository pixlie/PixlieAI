use crate::engine::{CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeId, NodeLabel, Payload};
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::web_page::WebPage;
use crate::error::{PiError, PiResult};
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
    pub path: String, // Relative to the domain
    pub query: Option<String>,
    pub is_fetched: bool,
}

impl Link {
    pub fn add(
        engine: Arc<&Engine>,
        url: &String,
        extra_labels: Vec<NodeLabel>,
        is_domain_allowed_to_crawl: bool,
    ) -> PiResult<NodeId> {
        // When we add a link to the graph, we check:
        // - if the domain already exists (no duplicates)
        // - if the path already exists
        // - if the query already exists
        // We do not store fragment
        // The link node only stores the path and query, domain is stored in the domain node
        match Url::parse(url) {
            Ok(parsed) => match parsed.domain() {
                Some(domain) => {
                    let domain_node_id = match Domain::get(
                        engine.clone(),
                        FindDomainOf::DomainName(domain.to_string()),
                    ) {
                        Ok((domain, domain_node_id)) => domain_node_id,
                        Err(err) => {
                            error!("Error getting domain node: {}", err);
                            Domain::add(
                                engine.clone(),
                                domain.to_string(),
                                extra_labels.clone(),
                                is_domain_allowed_to_crawl,
                            )?
                        }
                    };
                    let link_node_id = engine.add_node(
                        Payload::Link(Link {
                            path: parsed.path().to_string(),
                            query: parsed.query().map(|x| x.to_string()),
                            ..Default::default()
                        }),
                        extra_labels,
                    );
                    engine.add_connection(
                        (domain_node_id, link_node_id.clone()),
                        (
                            CommonEdgeLabels::SubPathOf.to_string(),
                            CommonEdgeLabels::RootPathOf.to_string(),
                        ),
                    );
                    Ok(link_node_id)
                }
                None => Err(PiError::InternalError(format!(
                    "Cannot parse domain from URL: {}",
                    url
                ))),
            },
            Err(_) => Err(PiError::InternalError(format!("Cannot parse URL: {}", url))),
        }
    }

    pub fn add_manually(engine: Arc<&Engine>, url: &String) -> PiResult<NodeId> {
        Self::add(
            engine.clone(),
            url,
            vec![CommonNodeLabels::AddedByUser.to_string()],
            true,
        )
    }

    pub fn get_full_link(&self) -> String {
        let mut url = self.path.clone();
        if let Some(query) = &self.query {
            url.push('?');
            url.push_str(query);
        }
        url
    }
}

impl Node for Link {
    fn get_label() -> String {
        "Link".to_string()
    }

    fn process(&self, engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<()> {
        // Download the linked URL and add a new WebPage node
        if self.is_fetched {
            return Ok(());
        }

        let node_id = node_id.clone();
        let url = self.get_full_link();
        let domain = Domain::get(engine.clone(), FindDomainOf::Node(node_id.clone()))?;
        let full_url = format!("https://{}{}", domain.0.name, url);
        let link = self.clone();
        let engine = engine.clone();
        debug!("Processing Link: {}", &full_url);

        thread::scope(|scope| {
            scope.spawn(|| match engine.fetch_url(full_url.clone(), &node_id) {
                Ok(rx) => match rx.recv() {
                    Ok(event) => match event {
                        FetchEvent::FetchResponse(_id, _url, contents) => {
                            debug!("Fetched HTML from {}", &full_url);
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
                                    CommonEdgeLabels::ContentOf.to_string(),
                                    CommonEdgeLabels::PathOf.to_string(),
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
                        error!("Can not fetch HTML from {}", &full_url);
                    }
                },
                Err(_err) => {
                    error!("Can not fetch HTML from {}", &full_url);
                }
            });
        });

        // Returning to the engine, the crawl will continue in the separate thread
        debug!("Returning from Link node: {}", self.path);
        Ok(())
    }
}
