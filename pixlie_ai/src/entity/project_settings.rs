// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeId, NodeItem, Payload};
use crate::engine::Engine;
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;
use url::Url;

// The project settings node contains high level settings that guide the flow of a project
#[derive(Clone, Default, Deserialize, Serialize, TS)]
pub struct ProjectSettings {
    pub only_extract_data_from_specified_links: bool,
    pub only_crawl_direct_links_from_specified_links: bool,
    pub only_crawl_within_domains_of_specified_links: bool,
}

impl ProjectSettings {
    pub fn can_scrape_link(
        &self,
        project_settings_node_id: &NodeId,
        link_node_id: &NodeId,
        current_link: &Url,
        link: &String,
        engine: Arc<&Engine>,
    ) -> PiResult<bool> {
        if self.only_extract_data_from_specified_links {
            return Ok(false);
        }

        if self.only_crawl_direct_links_from_specified_links {
            // If this link is connected to the ProjectSettings then we can scrape links from it
            let related_link_node_ids: Vec<NodeId> = engine.get_node_ids_connected_with_label(
                project_settings_node_id,
                &crate::engine::EdgeLabel::RelatedTo,
            )?;
            if related_link_node_ids.contains(link_node_id) {
                return Ok(true);
            }
        }

        if self.only_crawl_within_domains_of_specified_links {
            return if link.starts_with("/") {
                Ok(true)
            } else {
                match current_link.join(link) {
                    Ok(url) => {
                        let domain_of_link = url.domain().ok_or_else(|| {
                            PiError::InternalError(format!(
                                "Cannot parse URL {} to get domain",
                                &url
                            ))
                        })?;

                        // List the Links connected to the ProjectSettings node using the RelatedTo edge
                        let related_link_node_ids: Vec<NodeId> = engine
                            .get_node_ids_connected_with_label(
                                project_settings_node_id,
                                &crate::engine::EdgeLabel::RelatedTo,
                            )?;

                        let related_domain_nodes: Vec<(NodeId, NodeItem)> = related_link_node_ids
                            .iter()
                            .filter_map(|node_id| match engine.get_node_by_id(node_id) {
                                Some(node) => match &node.payload {
                                    Payload::Link(_) => {
                                        Link::get_domain_node(node_id, engine.clone())
                                            .unwrap_or_else(|_| None)
                                    }
                                    _ => None,
                                },
                                None => None,
                            })
                            .collect();

                        // Check if the domain is in the list of related domains
                        Ok(related_domain_nodes.iter().any(|(_, related_domain_node)| {
                            match &related_domain_node.payload {
                                Payload::Text(related_domain) => domain_of_link == related_domain,
                                _ => false,
                            }
                        }))
                    }
                    Err(_) => Ok(false),
                }
            };
        }

        Ok(false)
    }
}
