use crate::{
    config::get_cli_settings,
    engine::{Engine, NodeId, NodeWorker, Payload},
    entity::content::{Heading, Paragraph, TableCellType, TableRow, Title},
    error::PiResult,
    services::{anthropic, gliner, ollama, EntityExtractionProvider, TextClassificationProvider},
};
use log::{error, info};
use reqwest::blocking::get;
use scraper::Html;
use serde::{Deserialize, Serialize};
use url::Url;

use super::content::{BulletPoints, OrderedPoints};

// A link that should fetch
#[derive(Clone, Default, Deserialize, Serialize, Eq, PartialEq)]
pub struct Link {
    pub url: String,
    pub text: String,
    pub is_fetched: bool,
}

#[derive(Deserialize, Serialize, Eq, PartialEq)]
pub struct Domain(pub String);

impl NodeWorker for Link {
    fn process(&self, engine: &Engine, node_id: &NodeId) -> Option<Link> {
        // Download the linked URL and add a new WebPage node
        if self.is_fetched {
            // info!("Link already fetched: {}", self.url);
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
                            ..Default::default()
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

#[derive(Clone, Default, Deserialize, Serialize)]
pub struct WebPage {
    pub contents: String,
    pub is_scraped: bool,
    pub is_classified: bool,
    pub is_extracted: bool, // Has entity extraction process been run on this page
}

impl WebPage {
    fn get_link(&self, engine: &Engine, node_id: &NodeId) -> Option<Link> {
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
    }

    fn scrape_helper(&self, current_link: &Link) -> Vec<Payload> {
        let current_url = Url::parse(&current_link.url).unwrap();
        let current_domain = current_url.domain().unwrap();
        let mut parts: Vec<Payload> = vec![];

        let document = Html::parse_document(&self.contents);
        let start_node = document.root_element();
        for child in start_node.descendent_elements() {
            match child.value().name() {
                "title" => {
                    parts.push(Payload::Title(Title(
                        child
                            .text()
                            .collect::<Vec<&str>>()
                            .join("")
                            .trim()
                            .to_string(),
                    )));
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    parts.push(Payload::Heading(Heading(
                        child
                            .text()
                            .collect::<Vec<&str>>()
                            .join("")
                            .trim()
                            .to_string(),
                    )));
                }
                "p" => {
                    parts.push(Payload::Paragraph(Paragraph(
                        child
                            .text()
                            .map(|x| x.to_string())
                            .collect::<Vec<String>>()
                            .join("")
                            .trim()
                            .to_string(),
                    )));
                }
                "ul" => {
                    let mut bullet_points: Vec<String> = vec![];
                    for list_item in child.descendent_elements() {
                        match list_item.value().name() {
                            "li" => {
                                let text = list_item
                                    .text()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                                    .trim()
                                    .to_string();
                                if !text.is_empty() {
                                    bullet_points.push(text);
                                }
                            }
                            _ => {}
                        }
                    }
                    parts.push(Payload::BulletPoints(BulletPoints(bullet_points)));
                }
                "ol" => {
                    let mut ordered_points: Vec<String> = vec![];
                    for list_item in child.descendent_elements() {
                        match list_item.value().name() {
                            "li" => {
                                let text = list_item
                                    .text()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                                    .trim()
                                    .to_string();
                                if !text.is_empty() {
                                    ordered_points.push(text);
                                }
                            }
                            _ => {}
                        }
                    }
                    parts.push(Payload::OrderedPoints(OrderedPoints(ordered_points)));
                }
                "a" => {
                    if child.value().attr("href").is_none() {
                        continue;
                    }

                    let mut url = format!("{}", child.value().attr("href").unwrap());
                    // Skip links to anchors
                    if url.starts_with("#") {
                        continue;
                    }
                    let link_text = child.text().collect::<Vec<&str>>().join("");
                    if link_text.is_empty() {
                        continue;
                    }
                    if url.starts_with("/") {
                        match current_url.join(&url) {
                            Ok(parsed) => url = parsed.to_string(),
                            Err(_) => match Url::parse(&url) {
                                Ok(parsed) => match parsed.domain() {
                                    Some(domain) => {
                                        // Check if link is on the same domain as the current link
                                        if domain != current_domain {
                                            error!(
                                                "Can not parse URL to get domain for link: {}",
                                                url
                                            );
                                            continue;
                                        }
                                    }
                                    None => {
                                        error!("Can not parse URL to get domain for link: {}", url);
                                        continue;
                                    }
                                },
                                Err(err) => {
                                    error!(
                                        "Can not parse URL to get domain for link: {}\n{}",
                                        url, err
                                    );
                                    continue;
                                }
                            },
                        }
                    }
                    parts.push(Payload::Link(Link {
                        url: url.to_string(),
                        text: link_text,
                        ..Default::default()
                    }));
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
                            // engine.add_part_node(node_id, Payload::Table(Table(head)));
                            // for row in body {
                            // engine.add_part_node(node_id, Payload::TableRow(row));
                            // }
                        }
                    }
                }
                _ => {}
            }
        }
        parts
    }

    fn scrape(&self, engine: &Engine, node_id: &NodeId) {
        // Find the Link node that is the parent of this WebPage node
        let current_link = self.get_link(engine, node_id);
        if current_link.is_none() {
            error!("Cannot find parent Link node for WebPage node");
            return;
        }
        let current_link = current_link.unwrap();
        let parts = self.scrape_helper(&current_link);
        for part in parts {
            engine.add_part_node(node_id, part);
        }
    }

    fn get_content(&self, engine: &Engine, node_id: &NodeId) -> String {
        let part_nodes = engine
            .nodes
            .get(node_id)
            .unwrap()
            .read()
            .unwrap()
            .part_node_ids
            .clone();

        part_nodes
            .iter()
            .filter_map(
                |nid| match engine.nodes.get(nid).unwrap().read().unwrap().payload {
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
            )
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn classify(&self, engine: &Engine, node_id: &NodeId) -> PiResult<()> {
        // Classify the web page using Anthropic
        let settings = get_cli_settings().unwrap();
        let content = self.get_content(engine, node_id);
        if content.is_empty() {
            return Ok(());
        }
        let labels: Vec<String> = serde_yaml::from_str(WEBPAGE_CLASSIFICATION_LABELS).unwrap();

        let classification = match settings.get_text_classification_provider()? {
            TextClassificationProvider::Ollama => {
                // Use Ollama
                ollama::classify(&content, &labels, settings.ollama_hosts.unwrap(), 8080)?
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

    fn extract_entities(&self, engine: &Engine, node_id: &NodeId) -> PiResult<()> {
        // A WebPage is scraped into many **part** nodes, mainly content nodes, like Title, Heading, Paragraph, etc.
        // We collect all these nodes from the engine and pass them to the entity extraction service
        let settings = get_cli_settings().unwrap();
        let content = self.get_content(engine, node_id);
        let labels: Vec<String> = serde_yaml::from_str(WEBPAGE_EXTRACTION_LABELS).unwrap();
        let _entities = match settings.get_entity_extraction_provider()? {
            EntityExtractionProvider::Gliner => {
                // Use GLiNER
                gliner::extract_entities(content, &labels)
            }
            EntityExtractionProvider::Ollama => {
                // Use Ollama
                ollama::extract_entities(content, &labels, settings.ollama_hosts.unwrap(), 8080)
            }
            EntityExtractionProvider::Anthropic => {
                // Use Anthropic
                anthropic::extract_entities(content, &labels, &settings.anthropic_api_key.unwrap())
            }
        }?;
        Ok(())
    }
}

impl NodeWorker for WebPage {
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
            let classification = engine
                .nodes
                .get(node_id)
                .unwrap()
                .read()
                .unwrap()
                .related_node_ids
                .iter()
                .find_map(|node_id| {
                    match engine.nodes.get(node_id).unwrap().read().unwrap().payload {
                        Payload::Label(ref label) => Some(label.clone()),
                        _ => None,
                    }
                });
            if classification.is_some_and(|x| x != "Other") {
                self.extract_entities(engine, node_id).unwrap();
                return Some(WebPage {
                    is_extracted: true,
                    ..self.clone()
                });
            }
        }
        None
    }
}

static WEBPAGE_EXTRACTION_LABELS: &str = r#"
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

#[cfg(test)]
mod tests {
    use super::*;
    use log::info;
    use std::{env::current_dir, fs::read_to_string};
    use test_log::test;

    #[test]
    fn test_webpage_scraper_basic_extraction() {
        let mut path = current_dir().unwrap();
        path.push("fixtures/test_pixlie_about_us.html");
        info!("Path: {}", path.display());
        let link = Link {
            url: "https://pixlie.com/about".to_string(),
            text: "Abount us".to_string(),
            ..Default::default()
        };
        let webpage = WebPage {
            contents: read_to_string(path).unwrap(),
            ..Default::default()
        };

        let parts = webpage.scrape_helper(&link);
        // Check page title
        assert_eq!(
            parts
                .iter()
                .filter(|payload| match payload {
                    Payload::Title(ref title) => {
                        title.0 == "About us - Pixlie"
                    }
                    _ => false,
                })
                .count(),
            1
        );
        // Count a few headings
        assert_eq!(
            parts
                .iter()
                .filter(|payload| match payload {
                    Payload::Heading(ref heading) => {
                        vec![
                            "Driven by Innovation, Powered by People",
                            "Our values",
                            "Privacy-First",
                            "Transparency",
                        ]
                        .contains(&heading.0.as_str())
                    }
                    _ => false,
                })
                .count(),
            4
        );
        // Count the number of links
        assert_eq!(
            parts
                .iter()
                .filter(|payload| match payload {
                    Payload::Link(_) => true,
                    _ => false,
                })
                .count(),
            11
        );
    }

    #[test]
    fn test_webpage_scraper_extract_content_list() {
        let mut path = current_dir().unwrap();
        path.push("fixtures/test_content_list.html");
        info!("Path: {}", path.display());
        let link = Link {
            url: "https://growthlist.co/category/startups/".to_string(),
            text: "Startups".to_string(),
            ..Default::default()
        };
        let webpage = WebPage {
            contents: read_to_string(path).unwrap(),
            ..Default::default()
        };

        let parts = webpage.scrape_helper(&link);
        // Check list item headings
        assert_eq!(
            parts
                .iter()
                .filter(|payload| match payload {
                    Payload::Heading(ref heading) => {
                        vec![
                            "List of Funded Healthcare Startups in NYC For 2024",
                            "40+ Startup Failure Statistics For 2024",
                            "List of Funded Mobile App Startups For 2024",
                            "List of Funded Legal Startups For 2024",
                        ]
                        .contains(&heading.0.as_str())
                    }
                    _ => false,
                })
                .count(),
            4
        );

        // Check index of a heading
        assert_eq!(
            parts.iter().position(|payload| match payload {
                Payload::Heading(ref heading) => {
                    heading.0.as_str() == "List of Funded Sports Startups For 2024"
                }
                _ => false,
            }),
            Some(48)
        );

        let part = parts.get(49);
        assert!(part.is_some());

        // Check the next element is the link after the above heading
        assert_eq!(
            match part.unwrap() {
                Payload::Link(ref link) => {
                    Some(link.url.clone())
                }
                _ => None,
            },
            Some("https://growthlist.co/sports-startups/".to_string())
        );

        let part = parts.get(50);
        assert!(part.is_some());

        // Check the next element is the date after the above heading
        assert_eq!(
            match part.unwrap() {
                Payload::Paragraph(ref paragraph) => {
                    Some(paragraph.0.clone())
                }
                _ => None,
            },
            Some("July 15, 2024October 26, 2023".to_string())
        );
    }
}
