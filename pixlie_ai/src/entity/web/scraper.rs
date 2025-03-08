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

#[cfg(test)]
mod tests {
    use super::super::fixtures::rlhfbook_index_page;
    use super::super::link::Link;
    use super::*;
    use crate::engine::engine::get_test_engine;
    use crate::engine::CommonNodeLabels;

    #[test]
    fn test_webpage_scraper_basic_extraction() {
        let test_engine = get_test_engine();
        let arced_test_engine = Arc::new(&test_engine);
        let link_node_id = Link::add_manually(
            arced_test_engine,
            &"https://rlhfbook.com/c/01-introduction.html".to_string(),
        )
        .unwrap();

        let webpage = WebPage(rlhfbook_index_page().to_string());
        let webpage_node_id = test_engine
            .get_or_add_node(
                Payload::FileHTML(webpage.clone()),
                vec![
                    CommonNodeLabels::Content.to_string(),
                    CommonNodeLabels::WebPage.to_string(),
                ],
                true,
                None,
            )
            .unwrap()
            .get_node_id();
        test_engine
            .add_connection(
                (link_node_id, webpage_node_id.clone()),
                (
                    CommonEdgeLabels::PathOf.to_string(),
                    CommonEdgeLabels::ContentOf.to_string(),
                ),
            )
            .unwrap();
        test_engine.process_nodes();

        let parent_of_webpage = test_engine
            .get_node_ids_connected_with_label(
                &webpage_node_id,
                &CommonEdgeLabels::ContentOf.to_string(),
            )
            .unwrap();
        assert_eq!(parent_of_webpage.len(), 1);

        let children_of_webpage = test_engine
            .get_node_ids_connected_with_label(
                &webpage_node_id,
                &CommonEdgeLabels::ParentOf.to_string(),
            )
            .unwrap();
        assert_eq!(children_of_webpage.len(), 64);

        let title_node = test_engine
            .get_node_by_id(children_of_webpage.first().unwrap())
            .unwrap();
        assert_eq!(
            match title_node.payload {
                Payload::Text(ref text) => text.as_str(),
                _ => "",
            },
            "Introduction | RLHF Book by Nathan Lambert"
        );
        assert_eq!(
            title_node.labels,
            vec![
                CommonNodeLabels::Title.to_string(),
                CommonNodeLabels::PartialContent.to_string(),
                "Text".to_string()
            ]
        );

        let heading_node = test_engine
            .get_node_by_id(children_of_webpage.get(1).unwrap())
            .unwrap();
        assert_eq!(
            match heading_node.payload {
                Payload::Text(ref text) => text.as_str(),
                _ => "",
            },
            "A Little Bit of Reinforcement Learning from Human Feedback"
        );

        let mut paragraph_nodes =
            test_engine.get_node_ids_with_label(&CommonNodeLabels::Paragraph.to_string());
        paragraph_nodes.sort();
        assert_eq!(paragraph_nodes.len(), 37);

        let paragraph = test_engine
            .get_node_by_id(paragraph_nodes.get(2).unwrap())
            .unwrap();
        assert_eq!(
            match paragraph.payload {
                Payload::Text(ref text) => text.as_str(),
                _ => "",
            },
            "Reinforcement learning from Human Feedback (RLHF) is a technique used to incorporate human information into AI systems. RLHF emerged primarily as a method to solve hard to specify problems. Its early applications were often in control problems and other traditional domains for reinforcement learning (RL). RLHF became most known through the release of ChatGPT and the subsequent rapid development of large language models (LLMs) and other foundation models."
        );

        let paragraph = test_engine
            .get_node_by_id(paragraph_nodes.get(4).unwrap())
            .unwrap();
        assert_eq!(
            match paragraph.payload {
                Payload::Text(ref text) => text.as_str(),
                _ => "",
            },
            "RLHF has been applied to many domains successfully, with complexity increasing as the techniques have matured. Early breakthrough experiments with RLHF were applied to deep reinforcement learning [1], summarization [2], following instructions [3], parsing web information for question answering [4], and “alignment” [5]."
        );
    }
}
