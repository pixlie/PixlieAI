// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::node::{NodeId, NodeItem, NodeLabel, Payload};
use crate::engine::{EdgeLabel, Engine};
use crate::entity::project_settings::ProjectSettings;
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
        .split_whitespace()
        .map(|x| x.to_string())
        .collect();
    text.join(" ")
}

struct Traverser<'a> {
    link_node_id: NodeId,
    webpage_node_id: NodeId,
    webpage_url: Url,
    arced_engine: Arc<&'a Engine>,
    project_settings: Option<(NodeId, ProjectSettings)>,
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
                "meta" => {
                    let value = element.value();
                    if let Some(content) = value.attr("content") {
                        if let Some(property) = value.attr("property") {
                            match property {
                                "og:site_name" => self.add_text_node(
                                    vec![NodeLabel::Name, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "og:url" => self.add_text_node(
                                    vec![NodeLabel::Url, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "og:title" => self.add_text_node(
                                    vec![NodeLabel::Title, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "og:description" => self.add_text_node(
                                    vec![NodeLabel::Description, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "og:image" => {
                                    if content.starts_with("http") {
                                        self.add_text_node(
                                            vec![NodeLabel::Image, NodeLabel::Metadata],
                                            content,
                                            false,
                                        )?;
                                    } else {
                                        let base_url = self.webpage_url.join("/")?;
                                        let content = base_url.join(content)?;
                                        self.add_text_node(
                                            vec![NodeLabel::Image, NodeLabel::Metadata],
                                            &content.to_string(),
                                            false,
                                        )?;
                                    }
                                }
                                "article:published_time" => self.add_text_node(
                                    vec![NodeLabel::CreatedAt, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "article:modified_time" => self.add_text_node(
                                    vec![NodeLabel::ModifiedAt, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "article:tag" => self.add_text_node(
                                    vec![NodeLabel::Tag, NodeLabel::Metadata],
                                    content,
                                    true,
                                )?,
                                _ => {}
                            }
                        }
                        if let Some(name) = value.attr("name") {
                            match name {
                                "twitter:title" => self.add_text_node(
                                    vec![NodeLabel::Title, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "twitter:description" => self.add_text_node(
                                    vec![NodeLabel::Description, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "twitter:image" => {
                                    if content.starts_with("http") {
                                        self.add_text_node(
                                            vec![NodeLabel::Image, NodeLabel::Metadata],
                                            content,
                                            false,
                                        )?;
                                    } else {
                                        let base_url = self.webpage_url.join("/")?;
                                        let content = base_url.join(content)?;
                                        self.add_text_node(
                                            vec![NodeLabel::Image, NodeLabel::Metadata],
                                            &content.to_string(),
                                            false,
                                        )?;
                                    }
                                }
                                "title" => self.add_text_node(
                                    vec![NodeLabel::Title, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "description" => self.add_text_node(
                                    vec![NodeLabel::Description, NodeLabel::Metadata],
                                    content,
                                    false,
                                )?,
                                "image" => {
                                    if content.starts_with("http") {
                                        self.add_text_node(
                                            vec![NodeLabel::Image, NodeLabel::Metadata],
                                            content,
                                            false,
                                        )?;
                                    } else {
                                        let base_url = self.webpage_url.join("/")?;
                                        let content = base_url.join(content)?;
                                        self.add_text_node(
                                            vec![NodeLabel::Image, NodeLabel::Metadata],
                                            &content.to_string(),
                                            false,
                                        )?;
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                "img" => {
                    if let Some(src) = element.value().attr("src") {
                        if src.starts_with("http") {
                            self.add_text_node(
                                vec![NodeLabel::Image, NodeLabel::Metadata],
                                src,
                                false,
                            )?;
                        } else {
                            let base_url = self.webpage_url.join("/")?;
                            let src = base_url.join(src)?;
                            self.add_text_node(
                                vec![NodeLabel::Image, NodeLabel::Metadata],
                                &src.to_string(),
                                false,
                            )?;
                        }
                    }
                }
                "link" => {
                    if let Some(href) = element.value().attr("href") {
                        if let Some(rel) = element.value().attr("rel") {
                            if rel.contains("icon") {
                                if href.starts_with("http") {
                                    self.add_text_node(
                                        vec![NodeLabel::Logo, NodeLabel::Metadata],
                                        href,
                                        false,
                                    )?;
                                } else {
                                    let base_url = self.webpage_url.join("/")?;
                                    let href = base_url.join(href)?;
                                    self.add_text_node(
                                        vec![NodeLabel::Logo, NodeLabel::Metadata],
                                        &href.to_string(),
                                        false,
                                    )?;
                                }
                            }
                        }
                    }
                }
                "title" => self.add_text_node(
                    vec![NodeLabel::Title, NodeLabel::Metadata],
                    &element.text().collect::<Vec<&str>>().join(""),
                    false,
                )?,
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
                        (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
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
                        (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
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
                        (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
                    )?;
                    if let Some(parent_node_id) = parent_node_id {
                        self.arced_engine.add_connection(
                            (parent_node_id, bullet_points_node_id.clone()),
                            (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
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
                        (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
                    )?;
                    if let Some(parent_node_id) = parent_node_id {
                        self.arced_engine.add_connection(
                            (parent_node_id, bullet_points_node_id.clone()),
                            (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
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
                                (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
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

                    match &self.project_settings {
                        Some((project_settings_node_id, project_settings)) => {
                            if !project_settings.can_scrape_link(
                                &project_settings_node_id,
                                &self.link_node_id,
                                &self.webpage_url,
                                &url.to_string(),
                                self.arced_engine.clone(),
                            )? {
                                continue;
                            }
                        }
                        None => {}
                    }

                    // We are only handling links which use https at this moment
                    let link_node_id = if url.starts_with("https://") {
                        // Check if ProjectSettings allows crawling links from other domains

                        // Links that are full URLs
                        match Link::add(
                            self.arced_engine.clone(),
                            &url,
                            vec![NodeLabel::Link],
                            vec![],
                            true,
                        ) {
                            Ok(link_node_id) => link_node_id,
                            Err(_) => {
                                continue;
                            }
                        }
                    } else {
                        // Links that are relative to this path or domain, we build the full URL
                        match self.webpage_url.join(&url) {
                            Ok(parsed) => match Link::add(
                                self.arced_engine.clone(),
                                &parsed.to_string(),
                                vec![NodeLabel::Link],
                                vec![],
                                false,
                            ) {
                                Ok(link_node_id) => link_node_id,
                                Err(_) => {
                                    continue;
                                }
                            },
                            Err(_) => {
                                continue;
                            }
                        }
                    };
                    self.arced_engine.add_connection(
                        (self.webpage_node_id, link_node_id.clone()),
                        (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
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
    fn node_label_already_exists(&self, label: &NodeLabel) -> PiResult<bool> {
        let child_ids = self
            .arced_engine
            .get_node_ids_connected_with_label(&self.webpage_node_id, &EdgeLabel::ParentOf)?;

        Ok(child_ids.iter().any(|id| {
            self.arced_engine
                .get_node_by_id(id)
                .map(|node| node.labels.contains(label))
                .unwrap_or(false)
        }))
    }
    fn add_text_node(
        &self,
        labels: Vec<NodeLabel>,
        text: &str,
        can_add_multiple: bool,
    ) -> PiResult<()> {
        if !can_add_multiple {
            if self.node_label_already_exists(&labels[0])? {
                return Ok(());
            }
        }
        let text_node_id = self
            .arced_engine
            .get_or_add_node(
                Payload::Text(clean_text(text.to_string())),
                labels,
                true,
                None,
            )?
            .get_node_id();
        self.arced_engine.add_connection(
            (self.webpage_node_id.clone(), text_node_id.clone()),
            (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
        )?;
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

    // Get the ProjectSettings which is RelatedTo this Link node
    let project_settings_node_id = engine
        .get_node_ids_connected_with_label(&current_link_node_id.clone(), &EdgeLabel::RelatedTo)?;
    let project_settings: Option<(NodeId, ProjectSettings)> = if project_settings_node_id.is_empty()
    {
        None
    } else {
        project_settings_node_id
            .iter()
            .find_map(|node_id| match engine.get_node_by_id(&node_id) {
                Some(node) => match &node.payload {
                    Payload::ProjectSettings(payload) => Some((node_id.clone(), payload.clone())),
                    _ => None,
                },
                None => None,
            })
    };

    match &node.payload {
        Payload::Text(payload) => {
            if payload.contains("%PDF-") || payload.contains("trailer") || payload.contains("%%EOF")
            {
                log::warn!(
                    "Skipping non-HTML payload (likely a PDF) for node {:?}",
                    node.id
                );
                return Ok(());
            }
            let document = Html::parse_document(&payload);
            let traverser = Traverser {
                link_node_id: current_link_node_id.clone(),
                webpage_node_id: node.id.clone(),
                webpage_url: current_url,
                arced_engine: engine,
                project_settings,
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
