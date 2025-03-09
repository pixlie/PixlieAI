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

fn clean_text(text: String) -> String {
    let text: Vec<String> = text
        .trim()
        .replace("\n", " ")
        .replace("\t", " ")
        .split_whitespace()
        .map(|x| x.to_string())
        .collect();
    text.join(" ")
}

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
                            Payload::Text(clean_text(child.text().collect::<Vec<&str>>().join(""))),
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
                            Payload::Text(clean_text(child.text().collect::<Vec<&str>>().join(""))),
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
                            Payload::Text(clean_text(
                                child
                                    .text()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join(""),
                            )),
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
                    // let bullet_points_node_id = engine
                    //     .get_or_add_node(
                    //         Payload::Tree,
                    //         vec![
                    //             CommonNodeLabels::BulletPoints.to_string(),
                    //             CommonNodeLabels::PartialContent.to_string(),
                    //         ],
                    //         true,
                    //         None,
                    //     )?
                    //     .get_node_id();
                    // engine.add_connection(
                    //     (node_id.clone(), bullet_points_node_id),
                    //     (
                    //         CommonEdgeLabels::ParentOf.to_string(),
                    //         CommonEdgeLabels::ChildOf.to_string(),
                    //     ),
                    // )?;
                    // for list_item in child.child_elements() {
                    //     match list_item.value().name() {
                    //         "li" => {
                    //             let text = match list_item.text().take(1).collect::<Vec<&str>>().first() {
                    //                 Some(text) => {
                    //                     let text =clean_text(text.to_string());
                    //                     if !text.is_empty() {
                    //                 },
                    //                 None => continue,
                    //             };
                    //         }
                    //         _ => {}
                    //     }
                    // }
                }
                "ol" => {
                    // let mut ordered_points: Vec<String> = vec![];
                    // for list_item in child.descendent_elements() {
                    //     match list_item.value().name() {
                    //         "li" => {
                    //             let text = list_item
                    //                 .text()
                    //                 .map(|x| x.to_string())
                    //                 .collect::<Vec<String>>()
                    //                 .join("")
                    //                 .trim()
                    //                 .to_string();
                    //             if !text.is_empty() {
                    //                 ordered_points.push(text);
                    //             }
                    //         }
                    //         _ => {}
                    //     }
                    // }
                    // let ordered_points_node_id = engine
                    //     .get_or_add_node(
                    //         Payload::Tree(ordered_points),
                    //         vec![
                    //             CommonNodeLabels::OrderedPoints.to_string(),
                    //             CommonNodeLabels::PartialContent.to_string(),
                    //         ],
                    //         true,
                    //         None,
                    //     )?
                    //     .get_node_id();
                    // engine.add_connection(
                    //     (node_id.clone(), ordered_points_node_id),
                    //     (
                    //         CommonEdgeLabels::ParentOf.to_string(),
                    //         CommonEdgeLabels::ChildOf.to_string(),
                    //     ),
                    // )?;
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
                    if url.starts_with("https://") || url.starts_with("http://") {
                        // Links that are full URLs
                        match Link::add(engine.clone(), &url, vec![], vec![], true, false) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error adding link: {}", err);
                            }
                        }
                    } else {
                        // Links that are relative to this path or domain, we build the full URL
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
