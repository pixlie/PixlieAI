use crate::{
    config::Settings,
    engine::{CommonEdgeLabels, CommonNodeLabels, Engine, Node, NodeId, Payload},
    error::PiResult,
    services::{anthropic, ollama, TextClassificationProvider},
};
use log::{debug, error, info};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use ts_rs::TS;
use url::Url;

mod fetcher;
mod scraper;

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

// A link that should fetch
#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq, TS)]
pub struct Link {
    pub url: String,
    pub is_fetched: bool,
}

impl Link {
    pub fn add(url: &String, engine: &Engine) -> PiResult<()> {
        match Url::parse(url) {
            Ok(parsed) => match parsed.domain() {
                Some(domain) => {
                    let link_node_id = engine.add_node(
                        Payload::Link(Link {
                            url: url.to_string(),
                            is_fetched: false,
                        }),
                        vec![CommonNodeLabels::AddedByUser.to_string()],
                    );
                    let domain_node_id = engine.add_node(
                        Payload::Domain(Domain {
                            name: domain.to_string(),
                            is_allowed_to_crawl: true,
                            last_fetched_at: None,
                        }),
                        vec![CommonNodeLabels::AddedByUser.to_string()],
                    );
                    engine.add_connection(
                        (link_node_id, domain_node_id),
                        (
                            CommonEdgeLabels::Related.to_string(),
                            CommonEdgeLabels::Related.to_string(),
                        ),
                    );
                }
                None => {
                    error!("Can not parse URL to get domain: {}", &url);
                }
            },
            Err(err) => match err {
                _ => {
                    error!("Can not parse URL to get domain: {}", &url);
                }
            },
        };
        Ok(())
    }
}

impl Node for Link {
    fn get_label() -> String {
        "Link".to_string()
    }

    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<Link> {
        debug!("Processing Link node: {}", self.url);
        // Download the linked URL and add a new WebPage node
        if self.is_fetched {
            return None;
        }

        if !engine.can_fetch_within_domain(node_id, self) {
            return None;
        }
        debug!("Domain for link {} is allowed to crawl", self.url);

        let client = match reqwest::blocking::Client::builder()
            .user_agent("Pixlie AI")
            .timeout(Duration::from_secs(10))
            .build()
        {
            Ok(client) => client,
            Err(err) => {
                error!("Error building reqwest client: {}", err);
                return None;
            }
        };
        match client.get(&self.url).send() {
            Ok(response) => match response.text() {
                Ok(contents) => {
                    debug!("Fetched HTML from {}", self.url);
                    let content_node_id = engine.add_node(
                        Payload::FileHTML(WebPage {
                            contents,
                            ..Default::default()
                        }),
                        vec![],
                    );
                    engine.add_connection(
                        (node_id.clone(), content_node_id),
                        (
                            CommonEdgeLabels::Content.to_string(),
                            CommonEdgeLabels::Path.to_string(),
                        ),
                    );
                    return Some(Link {
                        is_fetched: true,
                        ..self.clone()
                    });
                }
                Err(err) => {
                    error!("Error fetching link: {}", err);
                }
            },
            Err(err) => {
                error!("Error fetching link: {}", err);
            }
        }
        None
    }
}

#[derive(Clone, Default, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct WebPage {
    pub contents: String,
    pub is_scraped: bool,
    pub is_classified: bool,
    pub is_extracted: bool, // Has entity extraction process been run on this page
}

impl WebPage {
    fn get_link(&self, engine: &Engine, node_id: &NodeId) -> Option<Link> {
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
        let labels: Vec<String> = serde_yaml::from_str(WEBPAGE_CLASSIFICATION_LABELS).unwrap();

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

static WEBPAGE_CLASSIFICATION_LABELS: &str = r#"
[
    Startup,
    Funding,
    Investor,
    Founder,
    Product,
    Other,
]
"#;
