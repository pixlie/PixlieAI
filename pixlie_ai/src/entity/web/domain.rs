use crate::engine::{CommonEdgeLabels, Engine, Node, NodeId, NodeLabel, Payload};
use crate::error::{PiError, PiResult};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::fmt::format;
use std::sync::Arc;
use std::time::Instant;
use ts_rs::TS;
use url::Url;

#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, TS)]
#[ts(export)]
pub struct Domain {
    pub name: String,
    pub is_allowed_to_crawl: bool,
    #[ts(skip)]
    #[serde(skip)]
    pub last_fetched_at: Option<Instant>,
}

impl Node for Domain {
    fn get_label() -> String {
        "Domain".to_string()
    }
}

pub enum FindDomainOf {
    DomainName(String),
    Node(NodeId),
}

impl Domain {
    pub fn get(engine: Arc<&Engine>, data: FindDomainOf) -> PiResult<(Domain, NodeId)> {
        match data {
            FindDomainOf::DomainName(url) => match engine.node_ids_by_label.read() {
                Ok(node_ids_by_label) => match node_ids_by_label.get(&Domain::get_label()) {
                    Some(domain_node_ids) => match engine.nodes.read() {
                        Ok(nodes) => {
                            for node_id in domain_node_ids {
                                match nodes.get(node_id) {
                                    Some(node) => match node.read() {
                                        Ok(node) => match node.payload {
                                            Payload::Domain(ref domain) => {
                                                if domain.name == url {
                                                    return Ok((domain.clone(), node_id.clone()));
                                                }
                                            }
                                            _ => {}
                                        },
                                        Err(err) => {
                                            error!("Error reading node: {}", err);
                                        }
                                    },
                                    None => {
                                        error!("Cannot find node {}", node_id);
                                    }
                                }
                            }
                            Err(PiError::GraphError(format!(
                                "Cannot find domain node for {}",
                                url
                            )))
                        }
                        Err(err) => Err(PiError::InternalError(format!(
                            "Error reading nodes: {}",
                            err
                        ))),
                    },
                    None => Err(PiError::GraphError(format!(
                        "Cannot find domain node for {}",
                        url
                    ))),
                },
                Err(err) => Err(PiError::InternalError(format!(
                    "Error reading node_ids_by_label: {}",
                    err
                ))),
            },
            FindDomainOf::Node(node_id) => {
                let connected = engine.get_node_ids_connected_with_label(
                    &node_id,
                    &CommonEdgeLabels::RootPathOf.to_string(),
                )?;
                match engine.nodes.read() {
                    Ok(nodes) => {
                        for connected_node_id in connected {
                            match nodes.get(&connected_node_id) {
                                Some(node) => match node.read() {
                                    Ok(node) => match node.payload {
                                        Payload::Domain(ref domain) => {
                                            return Ok((domain.clone(), connected_node_id.clone()));
                                        }
                                        _ => {}
                                    },
                                    Err(err) => {
                                        error!("Error reading node: {}", err);
                                    }
                                },
                                None => {
                                    error!("Cannot find node {}", connected_node_id);
                                }
                            }
                        }
                        Err(PiError::GraphError(format!(
                            "Cannot find domain node for node {}",
                            node_id
                        )))
                    }
                    Err(err) => Err(PiError::InternalError(format!(
                        "Error reading nodes: {}",
                        err
                    ))),
                }
            }
        }
    }

    pub fn add(
        engine: Arc<&Engine>,
        domain: String,
        extra_labels: Vec<NodeLabel>,
        is_allowed_to_crawl: bool,
    ) -> PiResult<NodeId> {
        let domain_node_id = engine.add_node(
            Payload::Domain(Domain {
                name: domain.to_string(),
                is_allowed_to_crawl,
                last_fetched_at: None,
            }),
            extra_labels,
        );
        Ok(domain_node_id)
    }

    pub fn can_fetch_within_domain(
        engine: Arc<&Engine>,
        url: &String,
        node_id: &NodeId,
    ) -> PiResult<()> {
        // Get the related domain node for the URL from the engine
        // TODO: Move this function to the Domain node
        debug!("Checking if we can fetch within domain: {}", url);
        let (domain, domain_node_id): (Domain, NodeId) =
            Self::get(engine.clone(), FindDomainOf::Node(node_id.clone()))?;

        if !domain.is_allowed_to_crawl {
            error!("Domain is not allowed to crawl: {}", &domain.name);
            return Err(PiError::FetchError(
                "Domain is not allowed to crawl".to_string(),
            ));
        }

        // Check the last fetch time for this domain. We do not want to fetch too often.
        match domain.last_fetched_at {
            Some(start) => {
                if start.elapsed().as_secs() > 2 {
                    // We have fetched from this domain some time ago, we can fetch now
                } else {
                    // We have fetched from this domain very recently, we can not fetch now
                    return Err(PiError::FetchError(
                        "Domain was recently fetched from, cannot fetch now".to_string(),
                    ));
                }
            }
            None => {
                // We have not fetched from this domain before, we should fetch now
            }
        }

        // Update the domain at the domain node id
        match engine.nodes.read() {
            Ok(nodes) => match nodes.get(&domain_node_id) {
                Some(node) => match node.write() {
                    Ok(mut node) => {
                        node.payload = Payload::Domain(Domain {
                            name: domain.name.clone(),
                            is_allowed_to_crawl: true,
                            last_fetched_at: Some(Instant::now()),
                        });
                    }
                    Err(_err) => {
                        return Err(PiError::FetchError(
                            "Error writing to domain node".to_string(),
                        ));
                    }
                },
                None => {
                    return Err(PiError::FetchError(
                        "Cannot find domain node for link".to_string(),
                    ));
                }
            },
            Err(_err) => {
                return Err(PiError::FetchError("Error reading domain node".to_string()));
            }
        }

        debug!("Domain {} is allowed to crawl", domain.name);
        Ok(())
    }
}
