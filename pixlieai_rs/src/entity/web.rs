use crate::{
    config::get_cli_settings,
    engine::{Engine, NodeId, NodeWorker, Payload},
    entity::content::{Heading, Paragraph, Table, TableCellType, TableRow, Title},
    error::PiResult,
    services::{anthropic, gliner, EntityExtractionProvider, ExtractionRequest},
};
use log::{error, info};
use reqwest::blocking::get;
use scraper::Html;
use serde::{Deserialize, Serialize};
use url::{ParseError, Url};

#[derive(Clone, Deserialize, Serialize)]
pub struct Link {
    pub url: String,
    pub is_fetched: bool,
}

impl Link {
    pub fn new(url: String) -> Link {
        Link {
            url,
            is_fetched: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct Domain(pub String);

impl NodeWorker for Link {
    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<Link> {
        // Download the linked URL and add a new WebPage node
        if self.is_fetched {
            info!("Link already fetched: {}", self.url);
            return None;
        }
        // Get the domain for the URL
        match Url::parse(&self.url) {
            Ok(parsed) => match parsed.domain() {
                Some(domain) => {
                    engine.add_part_node(node_id, Payload::Domain(Domain(domain.to_string())));
                }
                None => {
                    error!("Can not parse URL to get domain for link: {}", self.url);
                    return None;
                }
            },
            Err(err) => match err {
                _ => {
                    error!("Can not parse URL to get domain for link: {}", self.url);
                    return None;
                }
            },
        }
        match get(&self.url) {
            Ok(response) => match response.text() {
                Ok(contents) => {
                    engine.add_related_node(
                        node_id,
                        Payload::FileHTML(WebPage {
                            contents,
                            is_scraped: false,
                            is_extracted: false,
                        }),
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

#[derive(Clone, Deserialize, Serialize)]
pub struct WebPage {
    pub contents: String,
    pub is_scraped: bool,
    pub is_extracted: bool, // Has entity extraction process been run on this page
}

impl WebPage {
    pub fn scrape(&self, engine: &Engine, node_id: &NodeId) {
        // Find the Link node that is the parent of this WebPage node
        let current_link = {
            let related_node_ids = engine
                .nodes
                .get(node_id)
                .unwrap()
                .read()
                .unwrap()
                .related_node_ids
                .clone();
            related_node_ids.iter().find_map(|node_id| {
                match engine.nodes.get(node_id).unwrap().read().unwrap().payload {
                    Payload::Link(ref link) => Some(link.clone()),
                    _ => None,
                }
            })
        };

        if current_link.is_none() {
            error!("Cannot find parent Link node for WebPage node");
            return;
        }
        let current_link = current_link.unwrap();
        let current_domain = match Url::parse(&current_link.url) {
            Ok(parsed) => match parsed.domain() {
                Some(domain) => domain.to_string(),
                None => {
                    error!(
                        "Can not parse URL to get domain for link: {}",
                        current_link.url
                    );
                    return;
                }
            },
            Err(err) => match err {
                _ => {
                    error!(
                        "Can not parse URL to get domain for link: {}",
                        current_link.url
                    );
                    return;
                }
            },
        };
        info!("Scraping URL {}", current_link.url);

        let mut links_found: Vec<String> = vec![];
        let mut titles_found: Vec<String> = vec![];
        let mut headings_found: Vec<String> = vec![];

        let document = Html::parse_document(&self.contents);
        let start_node = document.root_element();
        for child in start_node.descendent_elements() {
            match child.value().name() {
                "title" => {
                    titles_found.push(child.text().collect::<Vec<&str>>().join(""));
                    engine.add_part_node(
                        node_id,
                        Payload::Title(Title(
                            child
                                .text()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                                .trim()
                                .to_string(),
                        )),
                    );
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    headings_found.push(child.text().collect::<Vec<&str>>().join(""));
                    engine.add_part_node(
                        node_id,
                        Payload::Heading(Heading(
                            child
                                .text()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                                .trim()
                                .to_string(),
                        )),
                    );
                }
                "p" => {
                    // info!(
                    //     "Found paragraph: {}",
                    //     child.text().collect::<Vec<&str>>().join("")
                    // );
                    engine.add_part_node(
                        node_id,
                        Payload::Paragraph(Paragraph(
                            child
                                .text()
                                .map(|x| x.to_string())
                                .collect::<Vec<String>>()
                                .join("")
                                .trim()
                                .to_string(),
                        )),
                    );
                }
                "a" => {
                    let url = child.value().attr("href").unwrap();
                    // Skip links to anchors
                    if url.starts_with("#") {
                        continue;
                    }
                    let link_text = child.text().collect::<Vec<&str>>().join("");
                    if link_text.is_empty() {
                        continue;
                    }
                    match Url::parse(&url) {
                        Ok(parsed) => match parsed.domain() {
                            Some(domain) => {
                                // Check if link is on the same domain as the current link
                                if domain == current_domain {
                                    links_found.push(url.to_string());
                                    engine.add_related_node(
                                        node_id,
                                        Payload::Link(Link {
                                            url: child.value().attr("href").unwrap().to_string(),
                                            is_fetched: false,
                                        }),
                                    );
                                }
                            }
                            None => {
                                error!(
                                    "Can not parse URL to get domain for link: {}",
                                    child.value().attr("href").unwrap()
                                );
                            }
                        },
                        Err(err) => {
                            error!("Can not parse URL to get domain for link: {}\n{}", url, err);
                        }
                    }
                }
                "table" => {
                    let mut head: Vec<String> = vec![];
                    let mut body: Vec<TableRow> = vec![];
                    for table_child in child.descendent_elements() {
                        match table_child.value().name() {
                            "thead" => {
                                for table_head in table_child.descendent_elements() {
                                    match table_head.value().name() {
                                        "th" => {
                                            head.push(
                                                table_head
                                                    .text()
                                                    .map(|x| x.to_string())
                                                    .collect::<Vec<String>>()
                                                    .join("")
                                                    .trim()
                                                    .to_string(),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            "tbody" => {
                                for table_body in table_child.descendent_elements() {
                                    match table_body.value().name() {
                                        "tr" => {
                                            let mut row: Vec<TableCellType> = vec![];
                                            for table_cell in table_body.descendent_elements() {
                                                match table_cell.value().name() {
                                                    "td" => {
                                                        row.push(TableCellType::String(
                                                            table_cell
                                                                .text()
                                                                .map(|x| x.to_string())
                                                                .collect::<Vec<String>>()
                                                                .join("")
                                                                .trim()
                                                                .to_string(),
                                                        ));
                                                    }
                                                    _ => {}
                                                }
                                            }
                                            body.push(TableRow(row));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                    // We check that head and body are not empty and that the count of elements
                    // in head and in each row of body are the same
                    if !head.is_empty() && !body.is_empty() {
                        let len = head.len();
                        if body.iter().all(|row| row.0.len() == len) {
                            engine.add_part_node(node_id, Payload::Table(Table(head)));
                            for row in body {
                                engine.add_part_node(node_id, Payload::TableRow(row));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        info!("Links found: {}", links_found.join(", "));
        info!("Titles found: {}", titles_found.join(", "));
        info!("Headings found: {}", headings_found.join(", "));
    }
}

impl WebPage {
    pub fn extract_entities(&self, engine: &Engine, node_id: &NodeId) -> PiResult<()> {
        // A WebPage is scraped into many **part** nodes, mainly content nodes, like Title, Heading, Paragraph, etc.
        // We collect all these nodes from the engine and pass them to the entity extraction service
        let settings = get_cli_settings().unwrap();
        let part_nodes = engine
            .nodes
            .get(node_id)
            .unwrap()
            .read()
            .unwrap()
            .part_node_ids
            .clone();
        info!("Getting part nodes for web page");

        let content = part_nodes
            .iter()
            .filter_map(
                |nid| match engine.nodes.get(nid).unwrap().read().unwrap().payload {
                    Payload::Title(ref title) => Some(title.0.trim().to_string()),
                    Payload::Heading(ref heading) => Some(heading.0.trim().to_string()),
                    Payload::Paragraph(ref paragraph) => Some(paragraph.0.trim().to_string()),
                    _ => None,
                },
            )
            .collect::<Vec<String>>()
            .join("\n\n");
        // info!("Content to be sent for entity extraction: {}", content);
        let request = ExtractionRequest {
            text: content,
            labels: serde_yaml::from_str(WEBPAGE_EXTRACT_LABELS).unwrap(),
        };
        let entities = match settings.get_entity_extraction_provider()? {
            EntityExtractionProvider::Gliner => {
                // Use GLiNER
                gliner::extract_entities(request)
            }
            EntityExtractionProvider::Anthropic => {
                // Use Anthropic
                anthropic::extract_entities(request, &settings.anthropic_api_key.unwrap())
            }
        }?;
        info!(
            "Extracted entities {}",
            entities
                .iter()
                .map(|x| format!("{}: {}", x.label, x.matching_text))
                .collect::<Vec<String>>()
                .join("\n")
        );
        Ok(())
    }
}

impl NodeWorker for WebPage {
    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<WebPage> {
        if !self.is_scraped {
            self.scrape(engine, node_id);
            return Some(WebPage {
                is_scraped: true,
                ..self.clone()
            });
        } else if !self.is_extracted {
            self.extract_entities(engine, node_id).unwrap();
            return Some(WebPage {
                is_extracted: true,
                ..self.clone()
            });
        }
        None
    }
}

static WEBPAGE_EXTRACT_LABELS: &str = r#"
[
    Company,
    Funding,
    PreviousFunding,
    TotalFunding,
    Valuation,
    FundingStage,
    Investor,
    Founder,
]
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::get_test_settings,
        entity::content::Paragraph,
        services::{
            anthropic::{self, extract_entities},
            gliner, EntityExtractionProvider,
        },
    };
    use log::{error, info};
    use test_log::test;

    // fn test_extract_entities_from_startup_news() {
    //     let startup_news = CrawledWebPage {
    //         meta_keywords: vec![],
    //         meta_description: None,
    //         title: SAMPLE_NEWS_TITLE.to_string(),
    //         body: SAMPLE_NEWS_BODY.to_string(),
    //     };
    //     let settings = get_test_settings().unwrap();
    //     let provider: EntityExtractionProvider = EntityExtractionProvider::Gliner;

    //     let entities = match provider {
    //         EntityExtractionProvider::Gliner => {
    //             // Use GLiNER
    //             gliner::extract_entities(&startup_news, &settings.path_to_gliner_home).await
    //         }
    //         EntityExtractionProvider::Anthropic => {
    //             // Use Anthropic
    //             anthropic::extract_entities(&startup_news, &settings.anthropic_api_key).await
    //         }
    //     };

    //     match entities {
    //         Ok(entities) => {
    //             // Log the entities
    //             info!(
    //                 "Extracted entities:\n{}",
    //                 entities
    //                     .iter()
    //                     .map(|x| format!("{},{}", x.label, x.matching_text))
    //                     .collect::<Vec<String>>()
    //                     .join("\n")
    //             );
    //             assert!(entities.len() > 8);
    //             assert!(entities
    //                 .iter()
    //                 .any(|x| x.label == "Funding" && x.matching_text.contains("491M")));
    //         }
    //         Err(err) => {
    //             error!("Error extracting entities: {}", err);
    //         }
    //     };
    // }

    // #[test]
    // fn test_fetch_link() {
    //     let engine = Engine::new();
    //     engine.add_node(Payload::Link(Link {
    //         url: "http://localhost:4321/pixlieai-tests/webpage-with-table.html".to_string(),
    //         is_fetched: false,
    //     }));
    //     engine.process_nodes();
    //     engine.add_pending_nodes();
    //     let nodes = engine.nodes.read().unwrap();
    //     nodes.iter().for_each(|node| match node.payload {
    //         Payload::FileHTML(ref file_html) => {
    //             assert!(file_html.contents.contains("plantpathco.com"));
    //             assert!(file_html.contents.contains("Agrim Wholesale"));
    //         }
    //         _ => {}
    //     });
    // }

    // #[test]
    // fn test_webpage_worker() {
    //     let engine = Engine::new();
    //     let mut path = current_dir().unwrap();
    //     path.push("fixtures/test_webpage_with_table.html");
    //     info!("Path: {}", path.display());
    //     let contents = read_to_string(path).unwrap();
    //     engine.add_node(Payload::FileHTML(WebPage {
    //         contents,
    //         is_scraped: false,
    //         is_extracted: false,
    //     }));
    //     engine.process_nodes();
    //     engine.add_pending_nodes();

    //     let nodes = engine.nodes.read().unwrap();
    //     // Check page title
    //     assert_eq!(
    //         nodes
    //             .iter()
    //             .filter(|node| match node.payload {
    //                 Payload::Title(ref title) => {
    //                     title.0 == "List of The Latest Funded Startups For 2024 - Growth List"
    //                 }
    //                 _ => false,
    //             })
    //             .count(),
    //         1
    //     );
    //     // Count a few headings
    //     assert_eq!(
    //         nodes
    //             .iter()
    //             .filter(|node| match node.payload {
    //                 Payload::Heading(ref heading) => {
    //                     vec![
    //                         "Recently Funded Startups at a Glance",
    //                         "About The Author",
    //                         "Growth List Team",
    //                     ]
    //                     .contains(&heading.0.as_str())
    //                 }
    //                 _ => false,
    //             })
    //             .count(),
    //         3
    //     );
    //     // Count the number of tables
    //     assert_eq!(
    //         nodes
    //             .iter()
    //             .filter(|node| match node.payload {
    //                 Payload::Table(_) => true,
    //                 _ => false,
    //             })
    //             .count(),
    //         1
    //     );
    //     // Count the number of table rows
    //     assert_eq!(
    //         nodes
    //             .iter()
    //             .filter(|node| match node.payload {
    //                 Payload::TableRow(_) => true,
    //                 _ => false,
    //             })
    //             .count(),
    //         100
    //     );
    // }
}
