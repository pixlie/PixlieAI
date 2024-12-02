use super::{Link, WebPage};
use crate::engine::{Engine, NodeId, Payload};
use crate::entity::content::{BulletPoints, OrderedPoints};
use crate::entity::content::{Heading, Paragraph, TableCellType, TableRow, Title};
use log::error;
use scraper::Html;
use url::Url;

impl WebPage {
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

    pub fn scrape(&self, engine: &Engine, node_id: &NodeId) {
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
}

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
    fn test_webpage_scraper_heading_paragraph() {
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

        // Check that the paragraph follows the heading
        let heading_index = parts
            .iter()
            .position(|payload| match payload {
                Payload::Heading(ref heading) => {
                    heading.0.as_str() == "List of Funded Sports Startups For 2024"
                }
                _ => false,
            })
            .unwrap();
        let paragraph_index = parts
            .iter()
            .position(|payload| match payload {
                Payload::Paragraph(ref paragraph) => paragraph
                    .0
                    .contains("Check out our database of sports-related"),
                _ => false,
            })
            .unwrap();
        assert_eq!(heading_index + 3, paragraph_index);

        let part = parts.get(heading_index + 1);
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

        let part = parts.get(heading_index + 2);
        assert!(part.is_some());

        // Check the next element is the date after the above heading
        assert_eq!(
            match part.unwrap() {
                Payload::Paragraph(ref paragraph) => {
                    Some(paragraph.0.clone())
                }
                _ => None,
            },
            Some("October 26, 2023".to_string())
        );
    }
}
