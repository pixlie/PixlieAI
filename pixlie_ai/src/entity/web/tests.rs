// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

#[test]
fn test_webpage_scraper_rlhf_book() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{ArcedNodeItem, NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::web::link::Link;
    use std::fs::read_to_string;
    use std::path::Path;
    use std::sync::Arc;
    use url::Url;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);
    let link_node_id = Link::add(
        arced_test_engine,
        &"https://rlhfbook.com/c/01-introduction.html".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();

    let path_to_webpage = Path::new("fixtures/rlhf_book_intro.html");
    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(read_to_string(path_to_webpage).unwrap()),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();
    test_engine.process_nodes();

    let parent_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ContentOf)
        .unwrap();
    assert_eq!(parent_of_webpage.len(), 1);

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 82);

    let title_nodes: Vec<ArcedNodeItem> = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap()
        .into_iter()
        .filter_map(|id| test_engine.get_node_by_id(&id))
        .filter(|node| node.labels.contains(&NodeLabel::Title))
        .collect();
    assert_eq!(title_nodes.len(), 1);
    let title_node = title_nodes.first().unwrap();
    assert_eq!(
        match title_node.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "Introduction | RLHF Book by Nathan Lambert"
    );
    assert_eq!(
        title_node.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    let heading_nodes: Vec<ArcedNodeItem> = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap()
        .into_iter()
        .filter_map(|id| test_engine.get_node_by_id(&id))
        .filter(|node| node.labels.contains(&NodeLabel::Heading))
        .collect();
    assert_eq!(heading_nodes.len(), 17);
    let heading_node = heading_nodes.first().unwrap();
    assert_eq!(
        match heading_node.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "A Little Bit of Reinforcement Learning from Human Feedback"
    );

    let mut paragraph_nodes = test_engine.get_node_ids_with_label(&NodeLabel::Paragraph);
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

    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 15);

    let domain_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Domain);
    assert_eq!(domain_node_ids.len(), 8);

    let all_domain_nodes: Vec<ArcedNodeItem> = domain_node_ids
        .iter()
        .filter_map(|node_id| match test_engine.get_node_by_id(node_id) {
            Some(node) => {
                if node.labels.contains(&NodeLabel::Domain) {
                    match node.payload {
                        Payload::Text(_) => Some(node.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            None => None,
        })
        .collect();

    let some_links = [
        "https://huggingface.co/allenai/tulu-2-dpo-70b",
        "https://huggingface.co/allenai/OLMo-7B-Instruct",
        "https://huggingface.co/allenai/tulu-2-dpo-70b",
        "https://github.com/huggingface/trl",
    ];
    // Check that all the links in some_links are found in all_anchor_nodes
    let mut count_matches = 0;
    for some_link in some_links {
        match Url::parse(some_link) {
            Ok(some_link_url) => {
                let some_domain = some_link_url.domain().unwrap();
                let some_path = some_link_url.path();
                let some_query = some_link_url.query();

                // Check this domain exists as a node
                for domain_node in all_domain_nodes.iter() {
                    match &domain_node.payload {
                        Payload::Text(domain) => {
                            if some_domain == domain {
                                // Get the link node for this domain
                                let link_node_ids = test_engine
                                    .get_node_ids_connected_with_label(
                                        &domain_node.id,
                                        &EdgeLabel::OwnerOf,
                                    )
                                    .unwrap();
                                let link_nodes = link_node_ids
                                    .iter()
                                    .filter_map(|node_id| {
                                        match test_engine.get_node_by_id(node_id) {
                                            Some(node) => match node.payload {
                                                Payload::Link(_) => Some(node.clone()),
                                                _ => None,
                                            },
                                            None => None,
                                        }
                                    })
                                    .collect::<Vec<ArcedNodeItem>>();
                                for link_node in link_nodes {
                                    match &link_node.payload {
                                        Payload::Link(link) => {
                                            if some_path == link.path
                                                && some_query == link.query.as_deref()
                                            {
                                                count_matches += 1;
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(err) => {
                panic!("Error parsing URL: {}", err);
            }
        }
    }
    assert_eq!(count_matches, some_links.len());

    let mut unordered_points_node_ids =
        test_engine.get_node_ids_with_label(&NodeLabel::UnorderedPoints);
    unordered_points_node_ids.sort();
    assert_eq!(unordered_points_node_ids.len(), 3);

    let mut ordered_points_node_ids =
        test_engine.get_node_ids_with_label(&NodeLabel::OrderedPoints);
    ordered_points_node_ids.sort();
    assert_eq!(ordered_points_node_ids.len(), 6);

    let first_bullet_point_node = test_engine
        .get_node_by_id(unordered_points_node_ids.first().unwrap())
        .unwrap();
    assert_eq!(
        first_bullet_point_node.labels,
        vec![NodeLabel::UnorderedPoints, NodeLabel::Partial,]
    );

    let list_item_node_ids = test_engine
        .get_node_ids_connected_with_label(&first_bullet_point_node.id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(list_item_node_ids.len(), 2);
    let list_item_nodes = list_item_node_ids
        .iter()
        .map(|node_id| test_engine.get_node_by_id(node_id).unwrap())
        .collect::<Vec<ArcedNodeItem>>();
    assert_eq!(
        list_item_nodes
            .iter()
            .map(|x| match x.payload {
                Payload::Text(ref text) => text.as_str(),
                Payload::Tree => "_tree_",
                _ => "",
            })
            .collect::<Vec<_>>(),
        vec!["Introduction", "Bibliography"]
    );

    let second_bullet_point_node = test_engine
        .get_node_by_id(unordered_points_node_ids.get(1).unwrap())
        .unwrap();
    assert_eq!(
        second_bullet_point_node.labels,
        vec![NodeLabel::UnorderedPoints, NodeLabel::Partial,]
    );

    let list_item_node_ids = test_engine
        .get_node_ids_connected_with_label(&second_bullet_point_node.id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(list_item_node_ids.len(), 4);
    let list_item_nodes = list_item_node_ids
        .iter()
        .map(|node_id| test_engine.get_node_by_id(node_id).unwrap())
        .collect::<Vec<ArcedNodeItem>>();
    assert_eq!(
        list_item_nodes
            .iter()
            .map(|x| match x.payload {
                Payload::Text(ref text) => text.as_str(),
                Payload::Tree => "_tree_",
                _ => "",
            })
            .collect::<Vec<_>>(),
        vec![
            "What Does RLHF Do?",
            "How We Got Here",
            "Scope of This Book",
            "Future of RLHF"
        ]
    );

    let third_bullet_point_node = test_engine
        .get_node_by_id(unordered_points_node_ids.get(2).unwrap())
        .unwrap();
    assert_eq!(
        third_bullet_point_node.labels,
        vec![NodeLabel::UnorderedPoints, NodeLabel::Partial,]
    );

    let list_item_node_ids = test_engine
        .get_node_ids_connected_with_label(&third_bullet_point_node.id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(list_item_node_ids.len(), 4);
    let list_item_nodes = list_item_node_ids
        .iter()
        .map(|node_id| test_engine.get_node_by_id(node_id).unwrap())
        .collect::<Vec<ArcedNodeItem>>();
    assert_eq!(
        list_item_nodes
            .iter()
            .map(|x| match x.payload {
                Payload::Text(ref text) => text.as_str(),
                Payload::Tree => "_tree_",
                _ => "",
            })
            .collect::<Vec<_>>(),
        vec![
            "Chapter Summaries",
            "Target Audience",
            "How to Use This Book",
            "About the Author"
        ]
    );
}

#[test]
fn test_extraction_from_hn_homepage() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{ArcedNodeItem, NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::web::link::Link;
    use std::fs::read_to_string;
    use std::path::Path;
    use std::sync::Arc;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);
    let link_node_id = Link::add(
        arced_test_engine,
        &"https://news.ycombinator.com".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();
    let path_to_webpage = Path::new("fixtures/hn_homepage.html");
    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(read_to_string(path_to_webpage).unwrap()),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();
    test_engine.process_nodes();

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 223);

    let title_nodes: Vec<ArcedNodeItem> = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap()
        .into_iter()
        .filter_map(|id| test_engine.get_node_by_id(&id))
        .filter(|node| node.labels.contains(&NodeLabel::Title))
        .collect();
    assert_eq!(title_nodes.len(), 1);
    let title_node = title_nodes.first().unwrap();
    assert_eq!(
        match title_node.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "Hacker News"
    );
    assert_eq!(
        title_node.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    // Count the number of Link nodes
    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 222);
}

#[test]
fn test_extract_data_only_from_specified_links() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{ArcedNodeItem, NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::project_settings::ProjectSettings;
    use crate::entity::web::link::Link;
    use std::fs::read_to_string;
    use std::path::Path;
    use std::sync::Arc;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);

    let project_settings_node_id = arced_test_engine
        .get_or_add_node(
            Payload::ProjectSettings(ProjectSettings {
                only_extract_data_from_specified_links: true,
                ..Default::default()
            }),
            vec![NodeLabel::AddedByUser, NodeLabel::ProjectSettings],
            true,
            None,
        )
        .unwrap()
        .get_node_id();

    let link_node_id = Link::add(
        arced_test_engine.clone(),
        &"https://news.ycombinator.com".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();

    arced_test_engine
        .add_connection(
            (project_settings_node_id, link_node_id),
            (EdgeLabel::RelatedTo, EdgeLabel::RelatedTo),
        )
        .unwrap();

    let path_to_webpage = Path::new("fixtures/hn_homepage.html");
    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(read_to_string(path_to_webpage).unwrap()),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();
    test_engine.process_nodes();

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 2);

    let title_nodes: Vec<ArcedNodeItem> = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap()
        .into_iter()
        .filter_map(|id| test_engine.get_node_by_id(&id))
        .filter(|node| node.labels.contains(&NodeLabel::Title))
        .collect();
    assert_eq!(title_nodes.len(), 1);
    let title_node = title_nodes.first().unwrap();
    assert_eq!(
        match title_node.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "Hacker News"
    );
    assert_eq!(
        title_node.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    // Count the number of Link nodes
    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 1);
}

#[test]
fn test_crawl_within_domains_of_specified_links() {
    use crate::engine::engine::get_test_engine;
    use crate::engine::node::{ArcedNodeItem, NodeLabel, Payload};
    use crate::engine::EdgeLabel;
    use crate::entity::project_settings::ProjectSettings;
    use crate::entity::web::link::Link;
    use std::fs::read_to_string;
    use std::path::Path;
    use std::sync::Arc;

    let test_engine = get_test_engine();
    let arced_test_engine = Arc::new(&test_engine);

    let project_settings_node_id = arced_test_engine
        .get_or_add_node(
            Payload::ProjectSettings(ProjectSettings {
                only_crawl_within_domains_of_specified_links: true,
                ..Default::default()
            }),
            vec![NodeLabel::AddedByUser, NodeLabel::ProjectSettings],
            true,
            None,
        )
        .unwrap()
        .get_node_id();

    let link_node_id = Link::add(
        arced_test_engine.clone(),
        &"https://news.ycombinator.com".to_string(),
        vec![NodeLabel::AddedByUser, NodeLabel::Link],
        vec![],
        true,
    )
    .unwrap();

    arced_test_engine
        .add_connection(
            (project_settings_node_id, link_node_id),
            (EdgeLabel::RelatedTo, EdgeLabel::RelatedTo),
        )
        .unwrap();
    let path_to_webpage = Path::new("fixtures/hn_homepage.html");
    let webpage_node_id = test_engine
        .get_or_add_node(
            Payload::Text(read_to_string(path_to_webpage).unwrap()),
            vec![NodeLabel::Content, NodeLabel::WebPage],
            true,
            None,
        )
        .unwrap()
        .get_node_id();
    test_engine
        .add_connection(
            (link_node_id, webpage_node_id.clone()),
            (EdgeLabel::PathOf, EdgeLabel::ContentOf),
        )
        .unwrap();

    // We process only once, so scraped links are not fetched
    test_engine.process_nodes();

    let children_of_webpage = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap();
    assert_eq!(children_of_webpage.len(), 190);

    let title_nodes: Vec<ArcedNodeItem> = test_engine
        .get_node_ids_connected_with_label(&webpage_node_id, &EdgeLabel::ParentOf)
        .unwrap()
        .into_iter()
        .filter_map(|id| test_engine.get_node_by_id(&id))
        .filter(|node| node.labels.contains(&NodeLabel::Title))
        .collect();
    assert_eq!(title_nodes.len(), 1);
    let title_node = title_nodes.first().unwrap();
    assert_eq!(
        match title_node.payload {
            Payload::Text(ref text) => text.as_str(),
            _ => "",
        },
        "Hacker News"
    );
    assert_eq!(
        title_node.labels,
        vec![NodeLabel::Title, NodeLabel::Partial]
    );

    // Count the number of Link nodes
    let link_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Link);
    assert_eq!(link_node_ids.len(), 189);

    // Check that there is only one Domain node
    let domain_node_ids = test_engine.get_node_ids_with_label(&NodeLabel::Domain);
    assert_eq!(domain_node_ids.len(), 1);

    let edges_from_domain_node = test_engine
        .get_node_ids_connected_with_label(domain_node_ids.get(0).unwrap(), &EdgeLabel::OwnerOf)
        .unwrap();
    assert_eq!(edges_from_domain_node.len(), 189);
}
