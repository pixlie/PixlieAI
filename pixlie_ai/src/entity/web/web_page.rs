// Copyright 2025 Pixlie Web Solutions Pvt. Ltd. (India)
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::{CommonEdgeLabels, Engine, Node, NodeId, Payload};
use crate::entity::web::link::Link;
use crate::error::{PiError, PiResult};
// use crate::services::{anthropic, ollama, TextClassificationProvider};
use crate::ExternalData;
use log::error;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use ts_rs::TS;

#[derive(Clone, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct WebPage {
    pub contents: String,
    pub is_scraped: bool,
    pub is_classified: bool,
    pub is_extracted: bool, // Has entity extraction process been run on this page
}

impl WebPage {
    pub fn get_link(&self, engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<(Link, NodeId)> {
        // Each WebPage node has a parent Link node, if not this is an error
        let related_node_ids = engine
            .get_node_ids_connected_with_label(node_id, &CommonEdgeLabels::ContentOf.to_string())?;
        let first_related_node_id = related_node_ids.first().ok_or_else(|| {
            PiError::InternalError("No related node ids found for WebPage node".to_string())
        })?;

        match engine.get_node_by_id(first_related_node_id) {
            Some(node) => match node.payload {
                Payload::Link(ref link) => Ok((link.clone(), **first_related_node_id)),
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

    fn get_content(&self, engine: Arc<&Engine>, node_id: &NodeId) -> String {
        let part_node_ids = match engine
            .get_node_ids_connected_with_label(node_id, &CommonEdgeLabels::ChildOf.to_string())
        {
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
                // Payload::Title(ref title) => Some(title.0.to_string()),
                Payload::Text(ref heading) => {
                    if heading.len() > 20 {
                        content.push_str(&heading);
                    }
                }
                Payload::Text(ref paragraph) => {
                    if paragraph.len() > 200 {
                        content.push_str(&paragraph);
                    }
                }
                _ => {}
            }
        }
        content
    }

    fn classify(&self, engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<()> {
        // Classify the web page using Anthropic
        // let settings = Settings::get_cli_settings()?;
        let content = self.get_content(engine, node_id);
        if content.is_empty() {
            return Ok(());
        }
        // let labels: Vec<String> = vec![];

        // let classification = match settings.get_text_classification_provider()? {
        //     TextClassificationProvider::Ollama => {
        //         // Use Ollama
        //         ollama::classify(
        //             &content,
        //             &labels,
        //             settings
        //                 .ollama_hosts
        //                 .unwrap()
        //                 .choose(&mut rand::thread_rng())
        //                 .unwrap(),
        //             8080,
        //         )?
        //     }
        //     TextClassificationProvider::Anthropic => {
        //         // Use Anthropic
        //         anthropic::classify(&content, &labels, &settings.anthropic_api_key.unwrap())?
        //     }
        // };
        // Insert the classification into the engine
        // engine.add_related_node(node_id, Payload::Label(classification.clone()));
        // info!("Content: {}\n\nclassified as: {}", content, classification);

        Ok(())
    }

    fn extract_entities(&self, _engine: &Engine, _node_id: &NodeId) -> PiResult<()> {
        // A WebPage is scraped into many **part** nodes, mainly content nodes, like Title, Heading, Paragraph, etc.
        // We collect all these nodes from the engine and pass them to the entity extraction service
        // let settings = Settings::get_cli_settings()?;
        // let content = self.get_content(engine, node_id);
        // let labels: Vec<String> = serde_yaml::from_str(WEBPAGE_EXTRACTION_LABELS).unwrap();
        // let _entities = match settings.get_entity_extraction_provider()? {
        //     EntityExtractionProvider::Gliner => {
        //         // Use GLiNER
        //         gliner::extract_entities(content, &labels)
        //     }
        //     EntityExtractionProvider::Ollama => {
        //         // Use Ollama
        //         ollama::extract_entities(
        //             content,
        //             &labels,
        //             settings
        //                 .ollama_hosts
        //                 .unwrap()
        //                 .choose(&mut rand::thread_rng())
        //                 .unwrap(),
        //             8080,
        //         )
        //     }
        //     EntityExtractionProvider::Anthropic => {
        //         // Use Anthropic
        //         anthropic::extract_entities(content, &labels, &settings.anthropic_api_key.unwrap())
        //     }
        // }?;
        Ok(())
    }
}

impl Node for WebPage {
    fn get_label() -> String {
        "WebPage".to_string()
    }

    fn process(
        &self,
        engine: Arc<&Engine>,
        node_id: &NodeId,
        _data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        // TODO: save the scraped nodes to graph only if webpage is classified as important to us
        if !self.is_scraped {
            self.scrape(engine.clone(), node_id)?;
            engine.update_node(
                &node_id,
                Payload::FileHTML(WebPage {
                    is_scraped: true,
                    ..self.clone()
                }),
            )?;
        } else if !self.is_classified {
            // self.classify(engine.clone(), node_id).unwrap();
            engine.update_node(
                &node_id,
                Payload::FileHTML(WebPage {
                    is_classified: true,
                    ..self.clone()
                }),
            )?;
        } else if !self.is_extracted {
            // Get the related Label node and check that classification is not "Other"
            // let classification =
            //     match engine.nodes.read() {
            //         Ok(nodes) => match nodes.get(node_id) {
            //             Some(node) => node.read().unwrap().edges.iter().find_map(|node_id| {
            //                 match engine.nodes.read() {
            //                     Ok(nodes) => match nodes.get(node_id) {
            //                         Some(node) => match node.read() {
            //                             Ok(node) => match node.payload {
            //                                 Payload::Label(ref label) => Some(label.clone()),
            //                                 _ => None,
            //                             },
            //                             Err(_err) => None,
            //                         },
            //                         None => None,
            //                     },
            //                     Err(_err) => None,
            //                 }
            //             }),
            //             None => None,
            //         },
            //         Err(_err) => None,
            //     };
            //
            // if classification.is_some_and(|x| x != "Other") {
            //     self.extract_entities(engine, node_id).unwrap();
            //     return Some(WebPage {
            //         is_extracted: true,
            //         ..self.clone()
            //     });
            // }
        }
        Ok(())
    }
}
