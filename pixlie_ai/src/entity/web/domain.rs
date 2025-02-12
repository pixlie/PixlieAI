use crate::engine::{CommonEdgeLabels, Engine, Node, NodeId, Payload};
use crate::error::{PiError, PiResult};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use ts_rs::TS;

#[derive(Clone, Deserialize, Serialize, Eq, PartialEq, TS)]
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

impl Domain {
    pub fn can_fetch_within_domain(engine: &Engine, url: String, node_id: &NodeId) -> PiResult<()> {
        // Get the related domain node for the URL from the engine
        // TODO: Move this function to the Domain node
        debug!("Checking if we can fetch within domain: {}", url);
        let (domain, domain_node_id): (Domain, NodeId) = {
            let connected = engine.get_node_ids_connected_with_label(
                node_id.clone(),
                CommonEdgeLabels::Related.to_string(),
            );
            debug!(
                "Found {} connected nodes with edge label {}",
                connected.len(),
                CommonEdgeLabels::Related.to_string()
            );
            let found: Option<(Domain, NodeId)> = match engine.nodes.read() {
                Ok(nodes) => connected
                    .iter()
                    .find_map(|node_id| match nodes.get(node_id) {
                        Some(node) => match node.read().unwrap().payload {
                            Payload::Domain(ref domain) => Some((domain.clone(), node_id.clone())),
                            _ => None,
                        },
                        None => None,
                    }),
                Err(_err) => None,
            };
            match found {
                Some(found) => found,
                None => {
                    error!("Cannot find domain node for link: {}", url);
                    return Err(PiError::FetchError(
                        "Cannot find domain for link".to_string(),
                    ));
                }
            }
        };

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
