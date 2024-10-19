// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::{
    engine::{Engine, Node, NodeId, Payload},
    entity::{
        fetchable::FetchStatus,
        web::{Link, WebPage},
    },
};
use std::any::Any;

pub mod web;

pub async fn fetch_links(engine: &Engine) {
    // We create references to nodes labeled Link
    // First, we get the vector of the nodes by label Link
    // Then, we filter nodes by node.id that are in the vector we got
    let (url, node_id) = {
        let mut url: Option<String> = None;
        let mut node_id: Option<NodeId> = None;
        let nodes_by_label = engine.nodes_by_label.read().await;
        let link_nodes = nodes_by_label.get("Link");

        if link_nodes.is_none() {
            return;
        }

        let link_nodes = link_nodes.unwrap();

        for node in link_nodes {
            // We get the node from the engine
            let nodes = engine.nodes.read().await;
            let link = nodes
                .iter()
                .find(|x| x.id == *node && matches!(x.payload, Payload::Link(_)))
                .unwrap();

            // We download the linked URL
            match link.payload {
                Payload::Link(ref link) => {
                    match link.fetched {
                        FetchStatus::NotFetched => {
                            // Since downloads can take time, we release the locks
                            // Mark this Link as Fetching and proceed to download
                            let mut nodes = engine.nodes.write().await;
                            nodes.iter_mut().find(|x| x.id == *node).unwrap().payload =
                                Payload::Link(Link {
                                    fetched: FetchStatus::Fetching,
                                    ..link.clone()
                                });
                            url = Some(link.url.clone());
                            node_id = Some(*node);
                        }
                        _ => {
                            // We have either downloaded or are downloading, ignore
                        }
                    }
                }
                _ => {}
            }
        }
        (url, node_id)
    };

    match url {
        Some(url) => {}
        None => {}
    }
}

pub async fn scrape(engine: &Engine) {
    // We create references to nodes labeled WebPage
    // First, we get the vector of the nodes by label WebPage
    // Then, we filter nodes by node.id that are in the vector we got
    let nodes_by_label = engine.nodes_by_label.read().await;
    let webpage_nodes = nodes_by_label.get("WebPage");

    if webpage_nodes.is_none() {
        return;
    }

    let webpage_nodes = webpage_nodes.unwrap();

    for node in webpage_nodes {
        // We get the node from the engine
        let webpage = engine
            .nodes
            .read()
            .await
            .iter()
            .find(|x| x.id == *node && matches!(x.payload, Payload::WebPage(_)))
            .unwrap();
    }
}
