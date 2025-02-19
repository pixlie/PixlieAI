use crate::engine::{CommonEdgeLabels, Engine, Node, NodeId, Payload};
use crate::error::{PiError, PiResult};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use ts_rs::TS;

#[derive(Clone, Default, Deserialize, Serialize, TS)]
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

pub enum FindDomainOf<'a> {
    DomainName(&'a str),
    Node(NodeId),
}

impl Domain {
    pub fn find_existing(
        engine: Arc<&Engine>,
        data: FindDomainOf,
    ) -> PiResult<Option<(Domain, NodeId)>> {
        match data {
            FindDomainOf::DomainName(domain_name) => match engine.node_ids_by_label.read() {
                Ok(node_ids_by_label) => match node_ids_by_label.get(&Domain::get_label()) {
                    Some(domain_node_ids) => match engine.nodes.read() {
                        Ok(nodes) => {
                            for node_id in domain_node_ids {
                                match nodes.get(node_id) {
                                    Some(node) => match node.read() {
                                        Ok(node) => match node.payload {
                                            Payload::Domain(ref domain) => {
                                                if domain.name == domain_name {
                                                    return Ok(Some((
                                                        domain.clone(),
                                                        node_id.clone(),
                                                    )));
                                                }
                                            }
                                            _ => {}
                                        },
                                        Err(err) => {
                                            // TODO: Log these errors somewhere, they should not happen
                                            error!("Error reading node: {}", err);
                                        }
                                    },
                                    None => {
                                        // TODO: Log these errors somewhere, they should not happen
                                        error!("Cannot find node {}", node_id);
                                    }
                                }
                            }
                            Ok(None)
                        }
                        Err(err) => Err(PiError::InternalError(format!(
                            "Error reading nodes: {}",
                            err
                        ))),
                    },
                    None => Err(PiError::GraphError(format!(
                        "Cannot find domain node for {}",
                        domain_name
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
                    &CommonEdgeLabels::BelongsTo.to_string(),
                )?;
                match engine.nodes.read() {
                    Ok(nodes) => {
                        for connected_node_id in connected {
                            match nodes.get(&connected_node_id) {
                                Some(node) => match node.read() {
                                    Ok(node) => match node.payload {
                                        Payload::Domain(ref domain) => {
                                            return Ok(Some((
                                                domain.clone(),
                                                connected_node_id.clone(),
                                            )));
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

    pub fn can_fetch_within_domain(
        engine: Arc<&Engine>,
        url: &str,
        node_id: &NodeId,
    ) -> PiResult<(Domain, NodeId)> {
        // Get the related domain node for the URL from the engine
        // TODO: Move this function to the Domain node
        debug!("Checking if we can fetch within domain: {}", url);
        let existing_domain: Option<(Domain, NodeId)> =
            Self::find_existing(engine.clone(), FindDomainOf::Node(node_id.clone()))?;
        let (domain, domain_node_id) = match existing_domain {
            Some(existing_domain) => existing_domain,
            None => {
                error!("Cannot find domain node for URL {}", url);
                return Err(PiError::InternalError(format!(
                    "Cannot find domain node for URL {}",
                    url
                )));
            }
        };

        if !domain.is_allowed_to_crawl {
            debug!("Domain is not allowed to crawl: {}", &domain.name);
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
                    debug!("Domain was recently fetched from, cannot fetch now");
                    return Err(PiError::FetchError(
                        "Domain was recently fetched from, cannot fetch now".to_string(),
                    ));
                }
            }
            None => {
                // We have not fetched from this domain before, we should fetch now
            }
        }

        debug!("Domain {} is allowed to crawl", domain.name);
        Ok((domain, domain_node_id))
    }
}
