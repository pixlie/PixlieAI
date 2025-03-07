use crate::engine::{CommonEdgeLabels, CommonNodeLabels, Engine, NodeId, Payload};
use crate::entity::content::CellData;
use crate::entity::content::{TableRow, TypedData};
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::entity::web::web_page::WebPage;
use crate::error::{PiError, PiResult};
use log::error;
use scraper::Html;
use std::sync::Arc;
use url::Url;

impl WebPage {
    pub fn scrape(&self, engine: Arc<&Engine>, node_id: &NodeId) -> PiResult<()> {
        // Find the Link node that is the parent of this WebPage node
        let (current_link, current_link_node_id) = self.get_link(engine.clone(), node_id)?;
        let existing_domain =
            Domain::find_existing(engine.clone(), FindDomainOf::Node(current_link_node_id))?;
        if existing_domain.is_none() {
            error!(
                "Cannot find domain node for URL {}",
                &current_link.get_full_link()
            );
            return Err(PiError::InternalError(format!(
                "Cannot find domain node for URL {}",
                &current_link.get_full_link()
            )));
        }
        let existing_domain = existing_domain.unwrap();
        let (domain_payload, _domain_node_id) = match existing_domain.0.payload {
            Payload::Domain(ref payload) => (payload.clone(), existing_domain.1.clone()),
            _ => {
                error!("Expected Domain type payload");
                return Err(PiError::GraphError(
                    "Expected Domain type payload".to_string(),
                ));
            }
        };
        // TODO: Use protocol from original link instead of https only
        let full_url = format!(
            "https://{}{}",
            domain_payload.name,
            current_link.get_full_link()
        );
        let current_url = match Url::parse(&full_url) {
            Ok(url) => url,
            Err(err) => {
                error!("Cannot parse URL {}: {}", &full_url, err);
                return Err(PiError::InternalError(format!(
                    "Cannot parse URL {}: {}",
                    &full_url, err
                )));
            }
        };

        let document = Html::parse_document(&self.0);
        let start_node = document.root_element();
        for child in start_node.descendent_elements() {
            match child.value().name() {
                "title" => {
                    let title_node_id = engine
                        .get_or_add_node(
                            Payload::Text(
                                child
                                    .text()
                                    .collect::<Vec<&str>>()
                                    .join("")
                                    .trim()
                                    .to_string(),
                            ),
                            vec![
                                CommonNodeLabels::Title.to_string(),
                                CommonNodeLabels::PartialContent.to_string(),
                            ],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node_id.clone(), title_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let heading_node_id = engine
                        .get_or_add_node(
                            Payload::Text(
                                child
                                    .text()
                                    .collect::<Vec<&str>>()
                                    .join("")
                                    .trim()
                                    .to_string(),
                            ),
                            vec![
                                CommonNodeLabels::Heading.to_string(),
                                CommonNodeLabels::PartialContent.to_string(),
                            ],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node_id.clone(), heading_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                }
                "p" => {
                    let paragraph_node_id = engine
                        .get_or_add_node(
                            Payload::Text(
                                child
                                    .text()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join("")
                                    .trim()
                                    .to_string(),
                            ),
                            vec![
                                CommonNodeLabels::Paragraph.to_string(),
                                CommonNodeLabels::PartialContent.to_string(),
                            ],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node_id.clone(), paragraph_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
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
                    let bullet_points_node_id = engine
                        .get_or_add_node(
                            Payload::ArrayOfTexts(bullet_points),
                            vec![
                                CommonNodeLabels::BulletPoints.to_string(),
                                CommonNodeLabels::PartialContent.to_string(),
                            ],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node_id.clone(), bullet_points_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
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
                    let ordered_points_node_id = engine
                        .get_or_add_node(
                            Payload::ArrayOfTexts(ordered_points),
                            vec![
                                CommonNodeLabels::OrderedPoints.to_string(),
                                CommonNodeLabels::PartialContent.to_string(),
                            ],
                            true,
                            None,
                        )?
                        .get_node_id();
                    engine.add_connection(
                        (node_id.clone(), ordered_points_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                }
                "a" => {
                    if child.value().attr("href").is_none() {
                        continue;
                    }

                    let url = format!("{}", child.value().attr("href").unwrap());
                    // Skip links to anchors
                    if url.starts_with("#") {
                        continue;
                    }
                    let link_text = child.text().collect::<Vec<&str>>().join("");
                    if link_text.is_empty() {
                        continue;
                    }
                    if url.starts_with("/") {
                        // Links that are relative to this website, we build the full URL
                        match current_url.join(&url) {
                            Ok(parsed) => {
                                match Link::add(
                                    engine.clone(),
                                    &parsed.to_string(),
                                    vec![],
                                    vec![],
                                    false,
                                    false,
                                ) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        error!("Error adding link: {}", err);
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Cannot parse URL to get domain for link: {}", err);
                            }
                        }
                    } else if url.starts_with("https://") || url.starts_with("http://") {
                        // Links that are full URLs
                        match Link::add(engine.clone(), &url, vec![], vec![], true, false) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error adding link: {}", err);
                            }
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
                                            let mut row: Vec<TypedData> = vec![];
                                            for table_cell in table_body.descendent_elements() {
                                                match table_cell.value().name() {
                                                    "td" => {
                                                        row.push(TypedData::String(
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
                                            body.push(TableRow(
                                                row.into_iter()
                                                    .map(|x| CellData::TypedData(x))
                                                    .collect(),
                                            ));
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
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use log::info;
//     use std::{env::current_dir, fs::read_to_string};
//     use test_log::test;
//
//     #[test]
//     fn test_webpage_scraper_basic_extraction() {
//         let mut path = current_dir().unwrap();
//         path.push("fixtures/test_pixlie_about_us.html");
//         info!("Path: {}", path.display());
//         let link = Link {
//             url: "https://pixlie.com/about".to_string(),
//             ..Default::default()
//         };
//         let webpage = WebPage {
//             contents: read_to_string(path).unwrap(),
//             ..Default::default()
//         };
//
//         let parts = webpage.scrape(&link);
//         // Check page title
//         assert_eq!(
//             parts
//                 .iter()
//                 .filter(|payload| match payload {
//                     Payload::Title(ref title) => {
//                         title.0 == "About us - Pixlie"
//                     }
//                     _ => false,
//                 })
//                 .count(),
//             1
//         );
//         // Count a few headings
//         assert_eq!(
//             parts
//                 .iter()
//                 .filter(|payload| match payload {
//                     Payload::Heading(ref heading) => {
//                         vec![
//                             "Driven by Innovation, Powered by People",
//                             "Our values",
//                             "Privacy-First",
//                             "Transparency",
//                         ]
//                         .contains(&heading.0.as_str())
//                     }
//                     _ => false,
//                 })
//                 .count(),
//             4
//         );
//         // Count the number of links
//         assert_eq!(
//             parts
//                 .iter()
//                 .filter(|payload| match payload {
//                     Payload::Link(_) => true,
//                     _ => false,
//                 })
//                 .count(),
//             11
//         );
//     }
//
//     #[test]
//     fn test_webpage_scraper_heading_paragraph() {
//         let mut path = current_dir().unwrap();
//         path.push("fixtures/test_content_list.html");
//         info!("Path: {}", path.display());
//         let link = Link {
//             url: "https://growthlist.co/category/startups/".to_string(),
//             ..Default::default()
//         };
//         let webpage = WebPage {
//             contents: read_to_string(path).unwrap(),
//             ..Default::default()
//         };
//
//         let parts = webpage.get_page_parts(&link);
//         // Check list item headings
//         assert_eq!(
//             parts
//                 .iter()
//                 .filter(|payload| match payload {
//                     Payload::Heading(ref heading) => {
//                         vec![
//                             "List of Funded Healthcare Startups in NYC For 2024",
//                             "40+ Startup Failure Statistics For 2024",
//                             "List of Funded Mobile App Startups For 2024",
//                             "List of Funded Legal Startups For 2024",
//                         ]
//                         .contains(&heading.0.as_str())
//                     }
//                     _ => false,
//                 })
//                 .count(),
//             4
//         );
//
//         // Check that the paragraph follows the heading
//         let heading_index = parts
//             .iter()
//             .position(|payload| match payload {
//                 Payload::Heading(ref heading) => {
//                     heading.0.as_str() == "List of Funded Sports Startups For 2024"
//                 }
//                 _ => false,
//             })
//             .unwrap();
//         let paragraph_index = parts
//             .iter()
//             .position(|payload| match payload {
//                 Payload::Paragraph(ref paragraph) => paragraph
//                     .0
//                     .contains("Check out our database of sports-related"),
//                 _ => false,
//             })
//             .unwrap();
//         assert_eq!(heading_index + 3, paragraph_index);
//
//         let part = parts.get(heading_index + 1);
//         assert!(part.is_some());
//
//         // Check the next element is the link after the above heading
//         assert_eq!(
//             match part.unwrap() {
//                 Payload::Link(ref link) => {
//                     Some(link.url.clone())
//                 }
//                 _ => None,
//             },
//             Some("https://growthlist.co/sports-startups/".to_string())
//         );
//
//         let part = parts.get(heading_index + 2);
//         assert!(part.is_some());
//
//         // Check the next element is the date after the above heading
//         assert_eq!(
//             match part.unwrap() {
//                 Payload::Paragraph(ref paragraph) => {
//                     Some(paragraph.0.clone())
//                 }
//                 _ => None,
//             },
//             Some("October 26, 2023".to_string())
//         );
//     }
// }
