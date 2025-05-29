use crate::engine::node::{
    ArcedNodeItem, ExistingOrNewNodeId, NodeId, NodeItem, NodeLabel, Payload,
};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::error::{PiError, PiResult};
use crate::{ExternalData, FetchRequest};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Arc;
use ts_rs::TS;
use url::Url;
use utoipa::ToSchema;

// A link that should fetch
#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, ToSchema, TS)]
pub struct Link {
    pub path: String, // Relative to the domain
    pub query: Option<String>,
}

impl Link {
    pub fn add(
        engine: Arc<&Engine>,
        url: &String,
        labels: Vec<NodeLabel>,
        domain_extra_labels: Vec<NodeLabel>,
        should_add_new_domain: bool,
    ) -> PiResult<NodeId> {
        // When we add a link to the graph, we check:
        // - if the domain already exists (no duplicates)
        // - if the path already exists
        // - if the query already exists
        // We do not store fragment
        // The link node only stores the path and query, domain is stored in the domain node
        let parsed = Url::parse(url).map_err(|err| {
            PiError::GraphError(format!("Cannot parse URL {} to get domain: {}", &url, err))
        })?;
        let domain = parsed.domain().ok_or_else(|| {
            PiError::InternalError(format!("Cannot parse URL {} to get domain", &url))
        })?;

        // TODO: Remove this and add the Domain label from all the calling functions
        let domain_extra_labels = if domain_extra_labels.contains(&NodeLabel::DomainName) {
            domain_extra_labels
        } else {
            [domain_extra_labels, vec![NodeLabel::DomainName]].concat()
        };
        let domain_node_id: NodeId = engine
            .get_or_add_node(
                Payload::Text(domain.to_string()),
                domain_extra_labels,
                should_add_new_domain,
                None,
            )?
            .get_node_id();

        let link_node_id = engine
            .get_or_add_node(
                Payload::Link(Link {
                    path: parsed.path().to_string(),
                    query: parsed.query().map(|x| x.to_string()),
                    ..Default::default()
                }),
                labels,
                true,
                // Engine will find possible existing Link rooted to this domain
                Some(domain_node_id),
            )?
            .get_node_id();

        engine.add_connection(
            (domain_node_id, link_node_id.clone()),
            (EdgeLabel::OwnerOf, EdgeLabel::BelongsTo),
        )?;
        Ok(link_node_id)
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
        find_related_to: Option<NodeId>,
    ) -> PiResult<Option<ArcedNodeItem>> {
        let domain_node: ArcedNodeItem = match find_related_to {
            Some(node_id) => match engine.get_node_by_id(&node_id) {
                Some(node) => node,
                None => {
                    error!("Cannot find node with ID {} for URL {}", node_id, url);
                    return Err(PiError::InternalError(format!(
                        "Cannot find node with ID {} for URL {}",
                        node_id, url
                    )));
                }
            },
            None => match Url::parse(url) {
                // To find an existing link, we first find the existing domain node
                Ok(parsed) => match parsed.domain() {
                    Some(domain) => {
                        match Domain::find_existing(
                            engine.clone(),
                            FindDomainOf::DomainName(domain),
                        )? {
                            Some(domain_node) => domain_node,
                            None => {
                                error!("Cannot find exiting domain node for URL {}", url);
                                return Err(PiError::InternalError(format!(
                                    "Cannot find exiting domain node for URL {}",
                                    url
                                )));
                            }
                        }
                    }
                    None => {
                        error!("Cannot parse URL {} to get domain", url);
                        return Err(PiError::InternalError(format!(
                            "Cannot parse URL {} to get domain",
                            url
                        )));
                    }
                },
                Err(err) => {
                    error!("Cannot parse URL {} to get domain: {}", url, err);
                    return Err(PiError::InternalError(format!(
                        "Cannot parse URL {} to get domain: {}",
                        url, err
                    )));
                }
            },
        };

        let url: String = if domain_node.labels.contains(&NodeLabel::DomainName) {
            match domain_node.payload {
                Payload::Text(ref domain) => format!("https://{}{}", domain, url),
                _ => {
                    error!("Cannot find domain node for URL {}", &url);
                    return Err(PiError::InternalError(format!(
                        "Cannot find domain node for URL {}",
                        &url
                    )));
                }
            }
        } else {
            error!("Cannot find domain node for URL {}", url);
            return Err(PiError::InternalError(format!(
                "Cannot find domain node for URL {}",
                &url
            )));
        };

        // We found an existing domain node, now we check if the link exists
        // We match link node by path and query
        match Url::parse(&url) {
            // To find an existing link, we first find the existing domain node
            Ok(parsed) => {
                let path = parsed.path().to_string();
                let query = parsed.query().map(|q| q.to_string());

                // We get all node IDs connected with the domain node
                let connected_node_ids: Vec<NodeId> = match engine
                    .get_node_ids_connected_with_label(&domain_node.id, &EdgeLabel::OwnerOf)
                {
                    Ok(connected_node_ids) => connected_node_ids,
                    Err(err) => {
                        error!("Error getting connected node IDs: {}", err);
                        return Err(err);
                    }
                };

                for node_id in connected_node_ids {
                    if let Some(node) = engine.get_node_by_id(&node_id) {
                        match &node.payload {
                            Payload::Link(link) => {
                                if link.path == path && link.query == query {
                                    return Ok(Some(node));
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            Err(err) => {
                error!("Error parsing URL: {}", err);
                return Err(PiError::InternalError(format!(
                    "Error parsing URL: {}",
                    err
                )));
            }
        };
        Ok(None)
    }

    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        // Download the linked URL and add a new WebPage node
        let url = match &node.payload {
            Payload::Link(link) => link.get_full_link(),
            _ => {
                return Err(PiError::InternalError(format!(
                    "Expected Payload::Link, got {}",
                    node.payload.to_string()
                )));
            }
        };
        match data_from_previous_request {
            Some(external_data) => match external_data {
                ExternalData::Response(response) => {
                    // We have received the contents of the URL from the previous request
                    debug!("Fetched HTML from {}", &url);
                    let content_node_id = match engine.get_or_add_node(
                        Payload::Text(response.contents),
                        vec![NodeLabel::Content, NodeLabel::WebPage],
                        true,
                        None,
                    ) {
                        Ok(existing_or_new_node_id) => match existing_or_new_node_id {
                            ExistingOrNewNodeId::Existing(id) => id,
                            ExistingOrNewNodeId::New(id) => id,
                        },
                        Err(err) => {
                            error!("Error adding node: {}", err);
                            return Err(err);
                        }
                    };
                    engine.add_connection(
                        (node.id.clone(), content_node_id),
                        (EdgeLabel::PathOf, EdgeLabel::ContentOf),
                    )?;
                    engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)?;
                }
                ExternalData::Error(error) => {
                    error!(
                        "Error processing link {}({}): {}. The link will be attempted again later.",
                        &url, node.id, error.error
                    );
                    engine.toggle_flag(&node.id, NodeFlags::HAD_ERROR)?;
                }
            },
            None => engine.fetch(FetchRequest::new(node.id, &url))?,
        }
        Ok(())
    }

    pub fn get_domain_node(
        node_id: &NodeId,
        engine: Arc<&Engine>,
    ) -> PiResult<Option<(NodeId, NodeItem)>> {
        match engine.get_node_ids_connected_with_label(&node_id, &EdgeLabel::BelongsTo) {
            Ok(connected_node_ids) => {
                // Find the first domain node
                Ok(connected_node_ids.iter().find_map(|node_id| {
                    match engine.get_node_by_id(node_id) {
                        Some(node) => {
                            if node.labels.contains(&NodeLabel::DomainName) {
                                match &node.payload {
                                    Payload::Text(_) => {
                                        Some((node_id.clone(), node.deref().clone()))
                                    }
                                    _ => None,
                                }
                            } else {
                                None
                            }
                        }
                        None => None,
                    }
                }))
            }
            Err(_) => Ok(None),
        }
    }
}
