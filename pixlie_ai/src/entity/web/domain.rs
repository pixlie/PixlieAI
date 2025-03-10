use crate::engine::{
    ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeFlags,
    NodeId, Payload,
};
use crate::error::{PiError, PiResult};
use crate::ExternalData;
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

    fn process(
        &self,
        engine: Arc<&Engine>,
        node_id: &NodeId,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        match data_from_previous_request {
            Some(external_data) => match external_data {
                ExternalData::Response(response) => {
                    // We have received the contents of the `robots.txt` from the previous request
                    let content_node_id = engine
                        .get_or_add_node(
                            Payload::Text(response.contents),
                            vec![CommonNodeLabels::RobotsTxt.to_string()],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node_id.clone(), content_node_id),
                        (
                            CommonEdgeLabels::OwnerOf.to_string(),
                            CommonEdgeLabels::BelongsTo.to_string(),
                        ),
                    )?;
                    engine.toggle_flag(&node_id, NodeFlags::IS_PROCESSED)?;
                }
                ExternalData::Error(_error) => {
                    // TODO: Make sure to not save timeout or other errors as blank robots.txt
                    let content_node_id = engine
                        .get_or_add_node(
                            Payload::Text("".to_string()),
                            vec![CommonNodeLabels::RobotsTxt.to_string()],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node_id.clone(), content_node_id),
                        (
                            CommonEdgeLabels::OwnerOf.to_string(),
                            CommonEdgeLabels::BelongsTo.to_string(),
                        ),
                    )?;
                    engine.toggle_flag(&node_id, NodeFlags::IS_PROCESSED)?;
                }
            },
            None => engine.fetch("/robots.txt", &node_id)?,
        };
        Ok(())
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
                // TODO: Implement and use better graph query API: https://github.com/pixlie/PixlieAI/issues/90, point 2
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
                // TODO: Implement and use better graph query API: https://github.com/pixlie/PixlieAI/issues/90, point 1
                let belongs_to = engine.get_node_ids_connected_with_label(
                    &node_id,
                    &CommonEdgeLabels::BelongsTo.to_string(),
                )?;
                let first_belongs_to = belongs_to.first().ok_or_else(|| {
                    PiError::InternalError(format!(
                        "Could not find Domain node for given node with ID {}",
                        node_id
                    ))
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
}
