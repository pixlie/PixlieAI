use super::NodeWorker;
use crate::{
    engine::{Engine, NodeId, Payload},
    entity::{
        content::{Heading, Paragraph, Table, TableCellType, TableRow, Title},
        web::{Link, WebPage},
    },
    error::{PiError, PiResult},
};
use chrono::Utc;
use log::{error, info};
use reqwest::{blocking, IntoUrl};
use scraper::{Html, Node};

impl NodeWorker for Link {
    fn process(&mut self, engine: &Engine, node_id: &NodeId) {
        // Download the linked URL and add a new WebPage node
        if !self.is_fetched {
            match blocking::get(&self.url) {
                Ok(response) => match response.text() {
                    Ok(contents) => {
                        engine.add_related_node(
                            node_id,
                            Payload::FileHTML(WebPage {
                                contents,
                                is_scraped: false,
                            }),
                        );
                        self.is_fetched = true;
                    }
                    Err(err) => {
                        error!("Error fetching link: {}", err);
                    }
                },
                Err(err) => {
                    error!("Error fetching link: {}", err);
                }
            }
        }
    }
}

impl NodeWorker for WebPage {
    fn process(&mut self, engine: &Engine, node_id: &NodeId) {
        if !self.is_scraped {
            let document = Html::parse_document(&self.contents);
            let mut start_node = document.root_element();
            for child in start_node.descendent_elements() {
                match child.value().name() {
                    "title" => {
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env::current_dir, fs::read_to_string};
    use test_log::test;

    // #[test]
    // fn test_fetch_link() {
    //     let engine = Engine::new();
    //     engine.add_node(Payload::Link(Link {
    //         url: "https://growthlist.co/funded-startups/".to_string(),
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

    #[test]
    fn test_webpage_worker() {
        let engine = Engine::new();
        let mut path = current_dir().unwrap();
        path.push("fixtures/test_webpage_with_table.html");
        info!("Path: {}", path.display());
        let contents = read_to_string(path).unwrap();
        engine.add_node(Payload::FileHTML(WebPage {
            contents,
            is_scraped: false,
        }));
        engine.process_nodes();
        engine.add_pending_nodes();

        let nodes = engine.nodes.read().unwrap();
        // Check page title
        assert_eq!(
            nodes
                .iter()
                .filter(|node| match node.payload {
                    Payload::Title(ref title) => {
                        title.0 == "List of The Latest Funded Startups For 2024 - Growth List"
                    }
                    _ => false,
                })
                .count(),
            1
        );
        // Count a few headings
        assert_eq!(
            nodes
                .iter()
                .filter(|node| match node.payload {
                    Payload::Heading(ref heading) => {
                        vec![
                            "Recently Funded Startups at a Glance",
                            "About The Author",
                            "Growth List Team",
                        ]
                        .contains(&heading.0.as_str())
                    }
                    _ => false,
                })
                .count(),
            3
        );
        // Count the number of tables
        assert_eq!(
            nodes
                .iter()
                .filter(|node| match node.payload {
                    Payload::Table(_) => true,
                    _ => false,
                })
                .count(),
            1
        );
        // Count the number of table rows
        assert_eq!(
            nodes
                .iter()
                .filter(|node| match node.payload {
                    Payload::TableRow(_) => true,
                    _ => false,
                })
                .count(),
            100
        );
    }
}
