use crate::engine::node::{NodeId, NodeItem, NodeLabel, Payload};
use crate::engine::{CommonEdgeLabels, Engine};
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::entity::web::web_page::get_link_of_webpage;
use crate::error::{PiError, PiResult};
use log::error;
use scraper::{ElementRef, Html};
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

struct Traverser<'a> {
    webpage_node_id: NodeId,
    webpage_url: Url,
    arced_engine: Arc<&'a Engine>,
}

impl<'a> Traverser<'a> {
    fn traverse(
        &self,
        given_el: ElementRef,
        parent_node_id: Option<NodeId>,
        parent_node_label: Option<String>,
    ) -> PiResult<()> {
        for element in given_el.child_elements() {
            let name = element.value().name();
            match name {
                "title" => {
                    let title_node_id = self
                        .arced_engine
                        .get_or_add_node(
                            Payload::Text(clean_text(
                                element.text().collect::<Vec<&str>>().join(""),
                            )),
                            vec![NodeLabel::Title, NodeLabel::Partial],
                            true,
                            None,
                        )?
                        .get_node_id();
                    self.arced_engine.add_connection(
                        (self.webpage_node_id.clone(), title_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    let heading_node_id = self
                        .arced_engine
                        .get_or_add_node(
                            Payload::Text(clean_text(
                                element.text().collect::<Vec<&str>>().join(""),
                            )),
                            vec![NodeLabel::Heading, NodeLabel::Partial],
                            true,
                            None,
                        )?
                        .get_node_id();
                    self.arced_engine.add_connection(
                        (self.webpage_node_id.clone(), heading_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                }
                "p" => {
                    let paragraph_node_id = self
                        .arced_engine
                        .get_or_add_node(
                            Payload::Text(clean_text(
                                element
                                    .text()
                                    .map(|x| x.to_string())
                                    .collect::<Vec<String>>()
                                    .join(""),
                            )),
                            vec![NodeLabel::Paragraph, NodeLabel::Partial],
                            true,
                            None,
                        )?
                        .get_node_id();
                    self.arced_engine.add_connection(
                        (self.webpage_node_id.clone(), paragraph_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                }
                "ul" => {
                    if !element.has_children() {
                        continue;
                    }
                    let bullet_points_node_id = self
                        .arced_engine
                        .get_or_add_node(
                            Payload::Tree,
                            vec![NodeLabel::UnorderedPoints, NodeLabel::Partial],
                            true,
                            None,
                        )?
                        .get_node_id();
                    self.arced_engine.add_connection(
                        (self.webpage_node_id.clone(), bullet_points_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                    if let Some(parent_node_id) = parent_node_id {
                        self.arced_engine.add_connection(
                            (parent_node_id, bullet_points_node_id.clone()),
                            (
                                CommonEdgeLabels::ParentOf.to_string(),
                                CommonEdgeLabels::ChildOf.to_string(),
                            ),
                        )?;
                    }
                    self.traverse(
                        element,
                        Some(bullet_points_node_id),
                        Some(NodeLabel::UnorderedPoints.to_string()),
                    )?;
                }
                "ol" => {
                    if !element.has_children() {
                        continue;
                    }
                    let bullet_points_node_id = self
                        .arced_engine
                        .get_or_add_node(
                            Payload::Tree,
                            vec![NodeLabel::OrderedPoints, NodeLabel::Partial],
                            true,
                            None,
                        )?
                        .get_node_id();
                    self.arced_engine.add_connection(
                        (self.webpage_node_id.clone(), bullet_points_node_id),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                    if let Some(parent_node_id) = parent_node_id {
                        self.arced_engine.add_connection(
                            (parent_node_id, bullet_points_node_id.clone()),
                            (
                                CommonEdgeLabels::ParentOf.to_string(),
                                CommonEdgeLabels::ChildOf.to_string(),
                            ),
                        )?;
                    }
                }
                "li" => {
                    // List items are stored only when parent is present and is either an ordered or an unordered list
                    if parent_node_label == Some(NodeLabel::UnorderedPoints.to_string())
                        || parent_node_label == Some(NodeLabel::OrderedPoints.to_string())
                    {
                        if let Some(parent_node_id) = parent_node_id {
                            let list_item_node_id = self
                                .arced_engine
                                .get_or_add_node(
                                    Payload::Text(clean_text(
                                        element.text().take(1).collect::<Vec<&str>>().join(""),
                                    )),
                                    vec![NodeLabel::ListItem, NodeLabel::Partial],
                                    true,
                                    None,
                                )?
                                .get_node_id();
                            self.arced_engine.add_connection(
                                (parent_node_id, list_item_node_id.clone()),
                                (
                                    CommonEdgeLabels::ParentOf.to_string(),
                                    CommonEdgeLabels::ChildOf.to_string(),
                                ),
                            )?;
                        }
                    }
                }
                "a" => {
                    if element.value().attr("href").is_none() {
                        continue;
                    }

                    let url = format!("{}", element.value().attr("href").unwrap());
                    // Skip links to anchors on the same page
                    if url.starts_with("#") {
                        continue;
                    }
                    let link_text = element.text().collect::<Vec<&str>>().join("");
                    if link_text.is_empty() {
                        continue;
                    }
                    // We are only handling links which use https at this moment
                    let link_node_id = if url.starts_with("https://") {
                        // Links that are full URLs
                        Link::add(self.arced_engine.clone(), &url, vec![], vec![], true)?
                    } else {
                        // Links that are relative to this path or domain, we build the full URL
                        match self.webpage_url.join(&url) {
                            Ok(parsed) => Link::add(
                                self.arced_engine.clone(),
                                &parsed.to_string(),
                                vec![],
                                vec![],
                                false,
                            )?,
                            Err(err) => {
                                error!("Cannot parse URL to get domain for link: {}", err);
                                continue;
                            }
                        }
                    };
                    self.arced_engine.add_connection(
                        (self.webpage_node_id, link_node_id.clone()),
                        (
                            CommonEdgeLabels::ParentOf.to_string(),
                            CommonEdgeLabels::ChildOf.to_string(),
                        ),
                    )?;
                }
                // "table" => {
                //     let mut head: Vec<String> = vec![];
                //     let mut body: Vec<TableRow> = vec![];
                //     for table_child in element.descendent_elements() {
                //         match table_child.value().name() {
                //             "thead" => {
                //                 for table_head in table_child.descendent_elements() {
                //                     match table_head.value().name() {
                //                         "th" => {
                //                             head.push(
                //                                 table_head
                //                                     .text()
                //                                     .map(|x| x.to_string())
                //                                     .collect::<Vec<String>>()
                //                                     .join("")
                //                                     .trim()
                //                                     .to_string(),
                //                             );
                //                         }
                //                         _ => {}
                //                     }
                //                 }
                //             }
                //             "tbody" => {
                //                 for table_body in table_child.descendent_elements() {
                //                     match table_body.value().name() {
                //                         "tr" => {
                //                             let mut row: Vec<TypedData> = vec![];
                //                             for table_cell in table_body.descendent_elements() {
                //                                 match table_cell.value().name() {
                //                                     "td" => {
                //                                         row.push(TypedData::String(
                //                                             table_cell
                //                                                 .text()
                //                                                 .map(|x| x.to_string())
                //                                                 .collect::<Vec<String>>()
                //                                                 .join("")
                //                                                 .trim()
                //                                                 .to_string(),
                //                                         ));
                //                                     }
                //                                     _ => {}
                //                                 }
                //                             }
                //                             body.push(TableRow(
                //                                 row.into_iter()
                //                                     .map(|x| CellData::TypedData(x))
                //                                     .collect(),
                //                             ));
                //                         }
                //                         _ => {}
                //                     }
                //                 }
                //             }
                //             _ => {}
                //         }
                //     }
                //     // We check that head and body are not empty and that the count of elements
                //     // in head and in each row of body are the same
                //     if !head.is_empty() && !body.is_empty() {
                //         let len = head.len();
                //         if body.iter().all(|row| row.0.len() == len) {
                //             // engine.add_part_node(node_id, Payload::Table(Table(head)));
                //             // for row in body {
                //             // engine.add_part_node(node_id, Payload::TableRow(row));
                //             // }
                //         }
                //     }
                // }
                _ => {}
            }
            if element.has_children() {
                self.traverse(element, None, None)?;
            }
        }
        Ok(())
    }
}

pub fn scrape(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<()> {
    // Find the Link node that is the parent of this WebPage node
    let (current_link, current_link_node_id) = get_link_of_webpage(engine.clone(), &node.id)?;
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
    let domain_name = Domain::get_domain_name(&existing_domain.unwrap())?;
    // TODO: Use protocol from original link instead of https only
    let full_url = format!("https://{}{}", domain_name, current_link.get_full_link());
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

    match &node.payload {
        Payload::Text(payload) => {
            let document = Html::parse_document(&payload);
            let traverser = Traverser {
                webpage_node_id: node.id.clone(),
                webpage_url: current_url,
                arced_engine: engine,
            };
            traverser.traverse(document.root_element(), None, None)?;
        }
        _ => {
            return Err(PiError::InternalError(format!(
                "Expected Payload::FileHTML, got {}",
                node.payload.to_string()
            )));
        }
    }
    Ok(())
}
