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
        domain_extra_labels: Vec<NodeLabel>,
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
                    let domain_node_id = match Domain::find_existing(
                        engine.clone(),
                        FindDomainOf::DomainName(domain),
                    )? {
                        Some((domain, domain_node_id)) => domain_node_id,
                        None => {
                            debug!("Existing domain node not found, adding new one");
                            Domain::add(
                                engine.clone(),
                                domain.to_string(),
                                domain_extra_labels,
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
            vec![],
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

    pub(crate) fn find_existing(
        engine: Arc<&Engine>,
        url: &str,
    ) -> PiResult<Option<(Link, NodeId)>> {
        match Url::parse(url) {
            // To find an existing link, we first find the existing domain node
            Ok(parsed) => match parsed.domain() {
                Some(domain) => {
                    match Domain::find_existing(engine.clone(), FindDomainOf::DomainName(domain))? {
                        Some((_domain_node, _domain_node_id)) => {
                            // We found an existing domain node, now we check if the link exists
                            // We match link node by path and query
                            let path = parsed.path().to_string();
                            let query = parsed.query().map(|q| q.to_string());

                            match engine.node_ids_by_label.read() {
                                Ok(node_ids_by_label) => {
                                    match node_ids_by_label.get(&Link::get_label().to_string()) {
                                        Some(link_node_ids) => {
                                            for node_id in link_node_ids {
                                                match engine.get_node_by_id(node_id) {
                                                    Ok(node) => match node.payload {
                                                        Payload::Link(link) => {
                                                            if link.path == path
                                                                && link.query == query
                                                            {
                                                                return Ok(Some((
                                                                    link.clone(),
                                                                    node_id.clone(),
                                                                )));
                                                            }
                                                        }
                                                        _ => {}
                                                    },
                                                    Err(_) => {}
                                                }
                                            }
                                            Ok(None)
                                        }
                                        None => {
                                            error!("Could not read node_ids_by_label");
                                            Err(PiError::InternalError(
                                                "Could not read node_ids_by_label".to_string(),
                                            ))
                                        }
                                    }
                                }
                                Err(err) => {
                                    error!("Could not read node_ids_by_label: {}", err);
                                    Err(PiError::InternalError(format!(
                                        "Could not read node_ids_by_label: {}",
                                        err
                                    )))
                                }
                            }
                        }
                        None => {
                            error!("Cannot find exiting domain node for URL {}", url);
                            Ok(None)
                        }
                    }
                }
                None => {
                    error!("Cannot parse URL {} to get domain", url);
                    Err(PiError::InternalError(format!(
                        "Cannot parse URL {} to get domain",
                        url
                    )))
                }
            },
            Err(err) => {
                error!("Cannot parse URL {} to get domain: {}", url, err);
                Err(PiError::InternalError(format!(
                    "Cannot parse URL {} to get domain: {}",
                    url, err
                )))
            }
        }
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
        let (domain, _domain_node_id) =
            match Domain::find_existing(engine.clone(), FindDomainOf::Node(node_id.clone()))? {
                Some((domain, domain_node_id)) => (domain, domain_node_id),
                None => {
                    error!("Cannot find domain for link node {}", node_id);
                    return Err(PiError::InternalError(format!(
                        "Cannot find domain node for link node {} with URL {}",
                        node_id, url
                    )));
                }
            };
        let full_url = format!("https://{}{}", domain.name, url);
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
