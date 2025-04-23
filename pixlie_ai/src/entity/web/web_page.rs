// Copyright 2025 Pixlie Web Solutions Pvt. Ltd. (India)
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeId, NodeItem, Payload};
use crate::engine::{EdgeLabel, Engine, NodeFlags};
use crate::entity::web::link::Link;
use crate::entity::web::web_metadata::WebMetadata;
use crate::entity::web::scraper::scrape;
use crate::error::{PiError, PiResult};
use crate::ExternalData;
use log::error;
use std::sync::Arc;

pub struct WebPage;

pub fn get_link_of_webpage(engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<(Link, NodeId)> {
    // Each WebPage node has a parent Link node, if not this is an error
    let related_node_ids =
        engine.get_node_ids_connected_with_label(node_id, &EdgeLabel::ContentOf)?;
    let first_related_node_id = related_node_ids.first().ok_or_else(|| {
        PiError::InternalError("No related node ids found for WebPage node".to_string())
    })?;

    match engine.get_node_by_id(first_related_node_id) {
        Some(node) => match node.payload {
            Payload::Link(ref link) => Ok((link.clone(), *first_related_node_id)),
            _ => Err(PiError::GraphError(
                "Cannot find parent Link node for WebPage node".to_string(),
            )),
        },
        None => {
            return Err(PiError::InternalError(format!(
                "Node with id {} not found",
                first_related_node_id
            )))
        }
    }
}

pub fn get_metadata_of_webpage(engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<(WebMetadata, NodeId)> {
    // Each WebPage may have a child WebMetadata node
    let related_node_ids =
        engine.get_node_ids_connected_with_label(node_id, &EdgeLabel::ChildOf)?;
    let first_related_node_id = related_node_ids.first().ok_or_else(|| {
        PiError::InternalError("No related node ids found for WebPage node".to_string())
    })?;

    match engine.get_node_by_id(first_related_node_id) {
        Some(node) => match node.payload {
            Payload::WebMetadata(ref metadata) => Ok((metadata.clone(), *first_related_node_id)),
            _ => Err(PiError::GraphError(
                "Cannot find child WebMetadata node for WebPage node".to_string(),
            )),
        },
        None => {
            return Err(PiError::InternalError(format!(
                "Node with id {} not found",
                first_related_node_id
            )))
        }
    }
}

impl WebPage {
    fn _get_content(&self, engine: Arc<&Engine>, node_id: &NodeId) -> String {
        let part_node_ids =
            match engine.get_node_ids_connected_with_label(node_id, &EdgeLabel::ChildOf) {
                Ok(part_node_ids) => part_node_ids,
                Err(err) => {
                    error!("Error getting part node ids: {}", err);
                    return "".to_string();
                }
            };

        let mut content = String::new();
        for node_id in part_node_ids {
            let node = match engine.get_node_by_id(&node_id) {
                Some(node) => node,
                None => continue,
            };

            match node.payload {
                Payload::Text(ref text) => {
                    if text.len() > 0 {
                        content.push_str(&text);
                    }
                }
                _ => {}
            }
        }
        content
    }

    // pub fn get_partial_content_nodes(
    //     &self,
    //     engine: Arc<&Engine>,
    //     node_id: &NodeId,
    // ) -> PiResult<Vec<ArcedNodeItem>> {
    //     match engine
    //         .get_node_ids_connected_with_label(node_id, &EdgeLabel::ParentOf)
    //     {
    //         Ok(partial_content_node_ids) => Ok(partial_content_node_ids
    //             .iter()
    //             .filter_map(|partial_content_node_id| {
    //                 match engine.get_node_by_id(partial_content_node_id) {
    //                     Some(node) => {
    //                         if node.labels.contains(&NodeLabel::Partial) {
    //                             Some(node)
    //                         } else {
    //                             None
    //                         }
    //                     }
    //                     None => None,
    //                 }
    //             })
    //             .collect::<Vec<ArcedNodeItem>>()),
    //         Err(err) => {
    //             error!("Error getting partial content nodes: {}", err);
    //             Err(err)
    //         }
    //     }
    // }

    // fn _classify(&self, _engine: Arc<&Engine>, _node_id: &NodeId) -> PiResult<()> {
    //     Classify the web page using Anthropic
    //     let settings = Settings::get_cli_settings()?;
    //     let content = self.get_content(engine, node_id);
    //     if content.is_empty() {
    //         return Ok(());
    //     }
    //     let labels: Vec<String> = vec![];
    //
    //     let classification = match settings.get_text_classification_provider()? {
    //         TextClassificationProvider::Ollama => {
    //             // Use Ollama
    //             ollama::classify(
    //                 &content,
    //                 &labels,
    //                 settings
    //                     .ollama_hosts
    //                     .unwrap()
    //                     .choose(&mut rand::thread_rng())
    //                     .unwrap(),
    //                 8080,
    //             )?
    //         }
    //         TextClassificationProvider::Anthropic => {
    //             // Use Anthropic
    //             anthropic::classify(&content, &labels, &settings.anthropic_api_key.unwrap())?
    //         }
    //     };
    //     Insert the classification into the engine
    //     engine.add_related_node(node_id, Payload::Label(classification.clone()));
    //     info!("Content: {}\n\nclassified as: {}", content, classification);
    //
    //     Ok(())
    // }

    // fn _extract_entities(&self, _engine: &Engine, _node_id: &NodeId) -> PiResult<()> {
    //     A WebPage is scraped into many **part** nodes, mainly content nodes, like Title, Heading, Paragraph, etc.
    //     We collect all these nodes from the engine and pass them to the entity extraction service
    //     let settings = Settings::get_cli_settings()?;
    //     let content = self.get_content(engine, node_id);
    //     let labels: Vec<String> = serde_yaml::from_str(WEBPAGE_EXTRACTION_LABELS).unwrap();
    //     let _entities = match settings.get_entity_extraction_provider()? {
    //         EntityExtractionProvider::Gliner => {
    //             // Use GLiNER
    //             gliner::extract_entities(content, &labels)
    //         }
    //         EntityExtractionProvider::Ollama => {
    //             // Use Ollama
    //             ollama::extract_entities(
    //                 content,
    //                 &labels,
    //                 settings
    //                     .ollama_hosts
    //                     .unwrap()
    //                     .choose(&mut rand::thread_rng())
    //                     .unwrap(),
    //                 8080,
    //             )
    //         }
    //         EntityExtractionProvider::Anthropic => {
    //             // Use Anthropic
    //             anthropic::extract_entities(content, &labels, &settings.anthropic_api_key.unwrap())
    //         }
    //     }?;
    //     Ok(())
    // }

    pub fn process(
        node: &NodeItem,
        engine: Arc<&Engine>,
        _data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        scrape(node, engine.clone())?;
        engine.toggle_flag(&node.id, NodeFlags::IS_PROCESSED)
    }
}

// else if !self.is_classified {
//     // self.classify(engine.clone(), node_id).unwrap();
//     engine.update_node(
//         &node_id,
//         Payload::FileHTML(WebPage {
//             is_classified: true,
//             ..self.clone()
//         }),
//     )?;
// }
// else if !self.is_extracted {
//     Get the related Label node and check that classification is not "Other"
//     let classification =
//         match engine.nodes.read() {
//             Ok(nodes) => match nodes.get(node_id) {
//                 Some(node) => node.read().unwrap().edges.iter().find_map(|node_id| {
//                     match engine.nodes.read() {
//                         Ok(nodes) => match nodes.get(node_id) {
//                             Some(node) => match node.read() {
//                                 Ok(node) => match node.payload {
//                                     Payload::Label(ref label) => Some(label.clone()),
//                                     _ => None,
//                                 },
//                                 Err(_err) => None,
//                             },
//                             None => None,
//                         },
//                         Err(_err) => None,
//                     }
//                 }),
//                 None => None,
//             },
//             Err(_err) => None,
//         };
//
//     if classification.is_some_and(|x| x != "Other") {
//         self.extract_entities(engine, node_id).unwrap();
//         return Some(WebPage {
//             is_extracted: true,
//             ..self.clone()
//         });
//     }
// }
