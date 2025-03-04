use crate::engine::{
    ArcedNodeId, ArcedNodeItem, CommonEdgeLabels, CommonNodeLabels, Engine, ExistingOrNewNodeId,
    Node, NodeFlags, NodeId, NodeLabel, Payload,
};
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::web_page::WebPage;
use crate::error::{PiError, PiResult};
use crate::ExternalData;
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use url::Url;

// A link that should fetch
#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, TS)]
pub struct Link {
    pub path: String, // Relative to the domain
    pub query: Option<String>,
}

impl Link {
    pub fn add(
        engine: Arc<&Engine>,
        url: &String,
        extra_labels: Vec<NodeLabel>,
        domain_extra_labels: Vec<NodeLabel>,
        should_add_new_domain: bool,
        is_domain_allowed_to_crawl: bool,
    ) -> PiResult<NodeId> {
        // When we add a link to the graph, we check:
        // - if the domain already exists (no duplicates)
        // - if the path already exists
        // - if the query already exists
        // We do not store fragment
        // The link node only stores the path and query, domain is stored in the domain node
        let parsed = Url::parse(url).map_err(|err| {
            PiError::InternalError(format!(
                "1. Cannot parse URL {} to get domain: {}",
                &url, err
            ))
        })?;
        let domain = parsed.domain().ok_or_else(|| {
            PiError::InternalError(format!("2. Cannot parse URL {} to get domain", &url))
        })?;
        let domain_node_id: NodeId = engine
            .get_or_add_node(
                Payload::Domain(Domain {
                    name: domain.to_string(),
                    is_allowed_to_crawl: is_domain_allowed_to_crawl,
                }),
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
                extra_labels,
                true,
                // Engine will find possible existing Link rooted to this domain
                Some(domain_node_id),
            )?
            .get_node_id();
        engine.add_connection(
            (domain_node_id, link_node_id.clone()),
            (
                CommonEdgeLabels::OwnerOf.to_string(),
                CommonEdgeLabels::BelongsTo.to_string(),
            ),
        )?;
        Ok(link_node_id)
    }

    pub fn add_manually(engine: Arc<&Engine>, url: &String) -> PiResult<NodeId> {
        Self::add(
            engine.clone(),
            url,
            vec![CommonNodeLabels::AddedByUser.to_string()],
            vec![],
            true,
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
        find_related_to: Option<NodeId>,
    ) -> PiResult<Option<(ArcedNodeItem, ArcedNodeId)>> {
        let (domain_node, domain_node_id) = match find_related_to {
            Some(node_id) => match engine.get_node_by_id(&node_id) {
                Some(node) => (node, node_id),
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
                            Some((domain_node, domain_node_id)) => (domain_node, *domain_node_id),
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

        let url = match domain_node.payload {
            Payload::Domain(ref domain) => format!("https://{}{}", domain.name, url),
            _ => {
                error!("Cannot find domain node for URL {}", url);
                return Err(PiError::InternalError(format!(
                    "Cannot find domain node for URL {}",
                    url
                )));
            }
        };

        // We found an existing domain node, now we check if the link exists
        // We match link node by path and query
        match Url::parse(&url) {
            // To find an existing link, we first find the existing domain node
            Ok(parsed) => {
                let path = parsed.path().to_string();
                let query = parsed.query().map(|q| q.to_string());

                // We get all node IDs connected with the domain node
                let connected_node_ids: Vec<ArcedNodeId> = match engine
                    .get_node_ids_connected_with_label(
                        &domain_node_id,
                        &CommonEdgeLabels::OwnerOf.to_string(),
                    ) {
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
                                    return Ok(Some((node, node_id.clone())));
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
}

impl Node for Link {
    fn get_label() -> String {
        "Link".to_string()
    }

    fn process(
        &self,
        engine: Arc<&Engine>,
        node_id: &NodeId,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        // Download the linked URL and add a new WebPage node
        let url = self.get_full_link();
        match data_from_previous_request {
            Some(external_data) => match external_data {
                ExternalData::Response(response) => {
                    // We have received the contents of the URL from the previous request
                    debug!("Fetched HTML from {}", &url);
                    let content_node_id = match engine.get_or_add_node(
                        Payload::FileHTML(WebPage {
                            contents: response.contents,
                            ..Default::default()
                        }),
                        vec![],
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
                        (node_id.clone(), content_node_id),
                        (
                            CommonEdgeLabels::PathOf.to_string(),
                            CommonEdgeLabels::ContentOf.to_string(),
                        ),
                    )?;
                    engine.toggle_flag(&node_id, NodeFlags::IS_PROCESSED)?;
                }
                ExternalData::Error(_error) => {}
            },
            None => engine.fetch(&url, &node_id)?,
        }
        Ok(())
    }
}
