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
use crate::entity::web::web_metadata::WebMetadata;
use crate::entity::web::web_page::{get_link_of_webpage, get_metadata_of_webpage};
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

fn text_without_subtrees(element: &ElementRef) -> String {
    // This function ignores ul, ol and table elements and their subtree
    // while calculating the text.
    // This is done to exclude such subtree text from the final text of an element, that
    // is expected to have nested children that we are explicitly processing as trees.
    // The function currently supports only the <li> element
    // Elements for future support include all sub-elements of <table>
    if element.value().name() != "li" {
        return element.text().collect::<Vec<&str>>().join("");
    }
    let mut text: Vec<String> = vec![];
    for child in element.children() {
        let value = child.value();
        if value.is_text() {
            if let Some(elem_text) = value.as_text() {
                let elem_text = elem_text.trim();
                if !elem_text.is_empty() {
                    text.push(elem_text.to_string());
                }
            }
        } else if value.is_element() {
            let sub_element_ref = ElementRef::wrap(child);
            if let Some(sub_element_ref) = sub_element_ref {
                match sub_element_ref.value().name() {
                    "ul" | "ol" | "table" => {}
                    _ => {
                        text.push(text_without_subtrees(&sub_element_ref));
                    }
                }
            }
        }
    }
    text.join("")
}

struct Traverser<'a> {
    link_node_id: NodeId,
    web_metadata_node_id: NodeId,
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
            let mut already_traversed = false;
            let name = element.value().name();
            match name {
                "meta" => {
                    if let Some(content) = element.value().attr("content") {
                        if let Some(property) = element.value().attr("property") {
                            self.update_metadata_node(&property, content)?;
                        }
                        if let Some(name) = element.value().attr("name") {
                            self.update_metadata_node(&name, content)?;
                        }
                    }
                }
                "link" => {
                    if let Some(href) = element.value().attr("href") {
                        if let Some(rel) = element.value().attr("rel") {
                            if rel.contains("icon") {
                                self.update_metadata_node("favicon", href)?;
                            }
                        }
                    }
                }
                "img" => {
                    if let Some(src) = element.value().attr("src") {
                        self.update_metadata_node("image", src)?;
                    }
                }
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
                        (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
                    )?;
                    self.update_metadata_node(
                        "title",
                        &clean_text(element.text().collect::<Vec<&str>>().join("")),
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
                        (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
                    )?;
                }
                "p" => {
                    let paragraph_node_id = self
                        .arced_engine
                        .get_or_add_node(
                            Payload::Text(clean_text(
                                element.text().collect::<Vec<&str>>().join(""),
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
                "ul" | "ol" => {
                    if !element.has_children() {
                        continue;
                    }
                    let labels = vec![
                        if name == "ul" {
                            NodeLabel::UnorderedPoints
                        } else {
                            NodeLabel::OrderedPoints
                        },
                        NodeLabel::Partial,
                    ];
                    let bullet_points_node_id = self
                        .arced_engine
                        .get_or_add_node(Payload::Tree, labels, true, None)?
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
                        Some(
                            (if name == "ul" {
                                NodeLabel::UnorderedPoints
                            } else {
                                NodeLabel::OrderedPoints
                            })
                            .to_string(),
                        ),
                    )?;
                    already_traversed = true;
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
                                    Payload::Text(clean_text(text_without_subtrees(&element))),
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
            if !already_traversed && element.has_children() {
                self.traverse(element, None, None)?;
            }
        }
        Ok(())
    }

    fn update_metadata_node(&self, attr: &str, content: &str) -> PiResult<()> {
        if content.is_empty() {
            return Ok(());
        }
        let node = self
            .arced_engine
            .get_node_by_id(&self.web_metadata_node_id)
            .ok_or_else(|| PiError::GraphError("WebMetadata node not found".into()))?;
        let mut payload = match &node.payload {
            Payload::WebMetadata(existing) => existing.clone(),
            _ => {
                return Err(PiError::InternalError(
                    "Node does not contain WebMetadata".into(),
                ))
            }
        };
        match attr.to_lowercase().as_str() {
            attr if attr.contains("url") => {
                if payload.url.is_some() {
                    return Ok(());
                }
                payload.url = Some(content.to_string())
            }
            attr if attr.contains("site_name") => payload.site_name = Some(content.to_string()),
            attr if attr.contains("favicon") => {
                if payload.favicon.is_some() {
                    return Ok(());
                }
                payload.favicon = Some(self.resolve_image(content)?)
            }
            attr if attr.contains("image") || attr.contains("thumbnail") => {
                if !attr.contains("width") && !attr.contains("height") {
                    return Ok(());
                }
                let is_probably_favicon = content.contains("fav")
                    || content.contains("apple-touch")
                    || content.contains(".ico");
                if is_probably_favicon {
                    if payload.favicon.is_some() {
                        return Ok(());
                    }
                    payload.favicon = Some(self.resolve_image(content)?)
                } else {
                    if payload.image.is_some() {
                        return Ok(());
                    }
                    payload.image = Some(self.resolve_image(content)?)
                }
            }
            attr if attr.contains("title") => {
                if payload.title.is_some() {
                    return Ok(());
                }
                payload.title = Some(content.to_string())
            }
            attr if attr.contains("description") => payload.description = Some(content.to_string()),
            attr if attr.contains("tags") || attr.contains("keywords") => payload
                .tags
                .get_or_insert_with(Vec::new)
                .push(content.to_string()),
            attr if attr.contains("published_time") => {
                payload.published_time = Some(content.to_string())
            }
            attr if attr.contains("modified_time") || attr.contains("updated_time") => {
                payload.modified_time = Some(content.to_string())
            }
            attr if attr.contains("author") => payload.author = Some(content.to_string()),
            attr if attr.contains("creator") => payload.creator = Some(content.to_string()),
            attr if attr.contains("lang") => payload.language = Some(content.to_string()),
            attr if attr.contains("locale") => payload.locale = Some(content.to_string()),
            _ => {
                return Ok(());
            }
        }
        let _ = self
            .arced_engine
            .update_node(&node.id, Payload::WebMetadata(payload));
        Ok(())
    }

    fn resolve_image(&self, content: &str) -> PiResult<String> {
        if content.starts_with("http") {
            Ok(content.to_string())
        } else {
            let base_url = self.webpage_url.join("/")?;
            Ok(base_url.join(content)?.to_string())
        }
    }
}

pub fn scrape(node: &NodeItem, engine: Arc<&Engine>) -> PiResult<()> {
    // Find the Link node that is the parent of this WebPage node
    let (current_link, current_link_node_id) = get_link_of_webpage(engine.clone(), &node.id)?;

    let web_metadata_node_id = match get_metadata_of_webpage(engine.clone(), &node.id) {
        Ok((_, id)) => id,
        Err(_) => {
            let new_id = engine
                .get_or_add_node(
                    Payload::WebMetadata(WebMetadata::default()),
                    vec![NodeLabel::WebMetadata],
                    true,
                    None,
                )?
                .get_node_id();

            engine.add_connection(
                (node.id.clone(), new_id.clone()),
                (EdgeLabel::ParentOf, EdgeLabel::ChildOf),
            )?;

            new_id
        }
    };

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
            if !payload.to_lowercase().contains("<html") {
                log::warn!(
                    "Skipping content that doesn't appear to be HTML for node {:?}",
                    node.id
                );
                return Ok(());
            }
            let document = Html::parse_document(&payload);
            let traverser = Traverser {
                link_node_id: current_link_node_id.clone(),
                web_metadata_node_id: web_metadata_node_id.clone(),
                webpage_node_id: node.id.clone(),
                webpage_url: current_url.clone(),
                arced_engine: engine,
                project_settings,
            };
            traverser.update_metadata_node("url", current_url.clone().as_str())?;
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
