use crate::config::Settings;
use crate::engine::{CommonEdgeLabels, Engine, Node, NodeId, Payload};
use crate::entity::web::link::Link;
use crate::error::PiResult;
use crate::services::{anthropic, ollama, TextClassificationProvider};
use log::info;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
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
    pub fn get_link(&self, engine: &Engine, node_id: &NodeId) -> Option<Link> {
        let related_node_ids = engine
            .get_node_ids_connected_with_label(node_id.clone(), CommonEdgeLabels::Path.to_string());

        related_node_ids
            .iter()
            .find_map(|node_id| match engine.nodes.read() {
                Ok(nodes) => match nodes.get(node_id) {
                    Some(node) => match node.read().unwrap().payload {
                        Payload::Link(ref link) => Some(link.clone()),
                        _ => None,
                    },
                    None => None,
                },
                Err(_err) => None,
            })
    }

    fn get_content(&self, engine: &Engine, node_id: &NodeId) -> String {
        let part_node_ids = engine.get_node_ids_connected_with_label(
            node_id.clone(),
            CommonEdgeLabels::Child.to_string(),
        );

        part_node_ids
            .iter()
            .filter_map(|nid| match engine.nodes.read() {
                Ok(nodes) => match nodes.get(nid) {
                    Some(node) => match node.read() {
                        Ok(node) => match node.payload {
                            // Payload::Title(ref title) => Some(title.0.to_string()),
                            Payload::Heading(ref heading) => {
                                if heading.0.len() > 20 {
                                    Some(heading.0.to_string())
                                } else {
                                    None
                                }
                            }
                            Payload::Paragraph(ref paragraph) => {
                                if paragraph.0.len() > 200 {
                                    Some(paragraph.0.to_string())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        },
                        Err(_err) => None,
                    },
                    None => None,
                },
                Err(_err) => None,
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn classify(&self, engine: &Engine, node_id: &NodeId) -> PiResult<()> {
        // Classify the web page using Anthropic
        let settings = Settings::get_cli_settings()?;
        let content = self.get_content(engine, node_id);
        if content.is_empty() {
            return Ok(());
        }
        let labels: Vec<String> = vec![];

        let classification = match settings.get_text_classification_provider()? {
            TextClassificationProvider::Ollama => {
                // Use Ollama
                ollama::classify(
                    &content,
                    &labels,
                    settings
                        .ollama_hosts
                        .unwrap()
                        .choose(&mut rand::thread_rng())
                        .unwrap(),
                    8080,
                )?
            }
            TextClassificationProvider::Anthropic => {
                // Use Anthropic
                anthropic::classify(&content, &labels, &settings.anthropic_api_key.unwrap())?
            }
        };
        // Insert the classification into the engine
        // engine.add_related_node(node_id, Payload::Label(classification.clone()));
        info!("Content: {}\n\nclassified as: {}", content, classification);

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

    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<WebPage> {
        // TODO: save the scraped nodes to graph only if webpage is classified as important to us
        if !self.is_scraped {
            self.scrape(engine, node_id);
            return Some(WebPage {
                is_scraped: true,
                ..self.clone()
            });
        } else if !self.is_classified {
            self.classify(engine, node_id).unwrap();
            return Some(WebPage {
                is_classified: true,
                ..self.clone()
            });
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
        None
    }
}
