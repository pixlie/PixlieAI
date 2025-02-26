use crate::engine::{ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, Engine, Node, NodeId, Payload};
use crate::error::{PiError, PiResult};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

#[derive(Clone, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct Domain {
    pub name: String,
    pub is_allowed_to_crawl: bool,
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
        find_type: FindDomainOf,
    ) -> PiResult<Option<(ArcedNodeItem, ArcedNodeId)>> {
        match find_type {
            FindDomainOf::DomainName(domain_name) => {
                let domain_node_ids = engine.get_node_ids_with_label(&Domain::get_label());
                for node_id in domain_node_ids {
                    match engine.get_node_by_id(&node_id) {
                        Some(node) => match node.payload {
                            Payload::Domain(ref domain) => {
                                if domain.name == domain_name {
                                    return Ok(Some((node, node_id)));
                                }
                            }
                            _ => {}
                        },
                        None => {}
                    }
                }
                Ok(None)
            }
            FindDomainOf::Node(node_id) => {
                let belongs_to = engine.get_node_ids_connected_with_label(
                    &node_id,
                    &CommonEdgeLabels::BelongsTo.to_string(),
                )?;
                let first_belongs_to = belongs_to.first().ok_or_else(|| {
                    PiError::InternalError(
                        "No connected node ids found for Domain node".to_string(),
                    )
                })?;
                match engine.get_node_by_id(first_belongs_to) {
                    Some(node) => match node.payload {
                        Payload::Domain(_) => Ok(Some((node, first_belongs_to.clone()))),
                        _ => Err(PiError::GraphError(
                            "Cannot find domain node for URL".to_string(),
                        )),
                    },
                    None => Err(PiError::InternalError(format!(
                        "Node with id {} not found",
                        first_belongs_to
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
        let existing_domain: Option<(ArcedNodeItem, ArcedNodeId)> =
            Self::find_existing(engine.clone(), FindDomainOf::Node(node_id.clone()))?;
        let (domain, _) = match existing_domain {
            Some(existing_domain) => existing_domain,
            None => {
                error!("Cannot find domain node for URL {}", url);
                return Err(PiError::InternalError(format!(
                    "Cannot find domain node for URL {}",
                    url
                )));
            }
        };

        match domain.payload {
            Payload::Domain(ref payload) => {
                if !payload.is_allowed_to_crawl {
                    error!("Domain is not allowed to crawl: {}", &payload.name);
                    return Err(PiError::FetchError(
                        "Domain is not allowed to crawl".to_string(),
                    ));
                }
                return Ok((payload.clone(), node_id.clone()));
            }
            _ => {}
        }

        Err(PiError::GraphError(format!(
            "Cannot find domain node for URL {}",
            url
        )))
    }
}
