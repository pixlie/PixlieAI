use crate::engine::node::{ArcedNodeItem, Fetchable, Node, NodeId, NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
use crate::error::{PiError, PiResult};
use crate::{ExternalData, FetchError, FetchRequest, FetchResponse};
use log::error;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::sync::Arc;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Domain {
    pub name: String,
}

impl Node for Domain {
    fn process(&self, node_id: NodeId, engine: Arc<&Engine>) -> PiResult<()> {
        // Initial processing logic - could trigger fetch requests
        let fetch_requests = self.make_fetch_requests(node_id, engine.clone())?;
        for request in fetch_requests {
            engine.fetch(request)?;
        }
        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn get_labels(&self) -> Vec<NodeLabel> {
        vec![NodeLabel::DomainName]
    }
}

impl Fetchable for Domain {
    fn make_fetch_requests(&self, node_id: NodeId, engine: Arc<&Engine>) -> PiResult<Vec<FetchRequest>> {
        // Check if we already have robots.txt
        // For now, always request it
        Ok(vec![FetchRequest::new(node_id, "/robots.txt")])
    }

    fn handle_fetch_response(&self, node_id: NodeId, engine: Arc<&Engine>, response: FetchResponse) -> PiResult<()> {
        // Handle robots.txt response
        let content_node_id = engine
            .get_or_add_node(
                Payload::Text(response.contents.clone()),
                vec![NodeLabel::RobotsTxt],
                true,
                None,
            )?
            .get_node_id();
        engine.add_connection(
            (node_id, content_node_id),
            (EdgeLabel::OwnerOf, EdgeLabel::BelongsTo),
        )?;
        engine.toggle_flag(&node_id, NodeFlags::IS_PROCESSED)?;
        Ok(())
    }

    fn handle_fetch_error(&self, node_id: NodeId, engine: Arc<&Engine>, error: FetchError) -> PiResult<()> {
        // Handle robots.txt fetch error - save empty robots.txt
        let content_node_id = engine
            .get_or_add_node(
                Payload::Text("".to_string()),
                vec![NodeLabel::RobotsTxt],
                true,
                None,
            )?
            .get_node_id();
        engine.add_connection(
            (node_id, content_node_id),
            (EdgeLabel::OwnerOf, EdgeLabel::BelongsTo),
        )?;
        engine.toggle_flag(&node_id, NodeFlags::IS_PROCESSED)?;
        Ok(())
    }
}

impl Domain {
    pub fn new(name: String) -> Self {
        Self { name }
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
    ) -> PiResult<Option<ArcedNodeItem>> {
        match find_type {
            FindDomainOf::DomainName(domain_name) => {
                // TODO: Implement and use better graph query API: https://github.com/pixlie/PixlieAI/issues/90, point 2
                let domain_node_ids = engine.get_node_ids_with_label(&NodeLabel::DomainName);
                for node_id in domain_node_ids {
                    match engine.get_node_by_id(&node_id) {
                        Some(node) => {
                            if node.labels.contains(&NodeLabel::DomainName) {
                                match node.payload {
                                    Payload::Text(ref domain) => {
                                        if domain == domain_name {
                                            return Ok(Some(node));
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        None => {}
                    }
                }
                Ok(None)
            }
            FindDomainOf::Node(node_id) => {
                // TODO: Implement and use better graph query API: https://github.com/pixlie/PixlieAI/issues/90, point 1
                let belongs_to =
                    engine.get_node_ids_connected_with_label(&node_id, &EdgeLabel::BelongsTo)?;
                let first_belongs_to = belongs_to.first().ok_or_else(|| {
                    PiError::InternalError(format!(
                        "Could not find Domain node for given node with ID {}",
                        node_id
                    ))
                })?;
                match engine.get_node_by_id(first_belongs_to) {
                    Some(node) => {
                        if node.labels.contains(&NodeLabel::DomainName) {
                            match node.payload {
                                Payload::Text(_) => Ok(Some(node)),
                                _ => Err(PiError::GraphError(
                                    "Cannot find domain node for URL".to_string(),
                                )),
                            }
                        } else {
                            Err(PiError::GraphError(
                                "Cannot find domain node for URL".to_string(),
                            ))
                        }
                    }
                    None => Err(PiError::InternalError(format!(
                        "Node with id {} not found",
                        first_belongs_to
                    ))),
                }
            }
        }
    }

    pub fn get_domain_name(node: &NodeItem) -> PiResult<String> {
        if !node.labels.contains(&NodeLabel::DomainName) {
            error!("Expected Domain type payload");
            return Err(PiError::GraphError(
                "Expected Domain type payload".to_string(),
            ));
        }
        match &node.payload {
            Payload::Text(data) => Ok(data.clone()),
            _ => {
                error!("Expected Domain type payload");
                Err(PiError::GraphError(
                    "Expected Domain type payload".to_string(),
                ))
            }
        }
    }

    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        match data_from_previous_request {
            Some(external_data) => match external_data {
                ExternalData::Response(response) => {
                    // We have received the contents of the `robots.txt` from the previous request
                    let content_node_id = engine
                        .get_or_add_node(
                            Payload::Text(response.contents.clone()),
                            vec![NodeLabel::RobotsTxt],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node.id.clone(), content_node_id),
                        (EdgeLabel::OwnerOf, EdgeLabel::BelongsTo),
                    )?;
                    engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                }
                ExternalData::Error(_error) => {
                    // TODO: Make sure to not save timeout or other errors as blank robots.txt
                    let content_node_id = engine
                        .get_or_add_node(
                            Payload::Text("".to_string()),
                            vec![NodeLabel::RobotsTxt],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node.id.clone(), content_node_id),
                        (EdgeLabel::OwnerOf, EdgeLabel::BelongsTo),
                    )?;
                    engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                }
            },
            None => engine.fetch(FetchRequest::new(node.id, "/robots.txt"))?,
        };
        Ok(())
    }
}
