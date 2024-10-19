// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use chrono::{DateTime, Utc};
use strum::Display;
// use petgraph::graph::NodeIndex;
// use petgraph::{Directed, Graph};
use crate::{
    entity::web::{Link, WebPage},
    workers,
};
use std::{
    any::Any,
    collections::{HashMap, HashSet},
};
use tokio::{
    sync::RwLock,
    time::{sleep, Duration},
};

pub mod api;
// pub mod executor;
// pub mod state;

#[derive(Display)]
pub enum Payload {
    WebPage(WebPage),
    Link(Link),
}

pub type NodeId = u32;
pub struct Node {
    pub id: NodeId,
    pub label: String,
    pub payload: Payload,
    pub written_at: DateTime<Utc>,
}

// The engine keeps track of all the data nodes and their relationships
// All the entity labels are loaded in the engine
// All data may not be loaded in the engine, some of them may be on disk
pub struct Engine {
    // pub graph: RwLock<Graph<PiNode, PiEdge, Directed>>,
    pub labels: RwLock<HashSet<String>>,
    pub nodes: RwLock<Vec<Node>>, // All nodes that are in the engine
    last_node_id: RwLock<u32>,
    // pub storage_root: String,
    pub nodes_by_label: RwLock<HashMap<String, Vec<NodeId>>>,
    // pub entity_type_last_run: RwLock<HashMap<String, DateTime<Utc>>>,
    pub execute_every: u8, // Number of seconds to wait before executing the engine
}

impl Engine {
    pub fn new() -> Engine {
        Engine {
            labels: RwLock::new(HashSet::new()),
            nodes: RwLock::new(vec![]),
            last_node_id: RwLock::new(0),
            nodes_by_label: RwLock::new(HashMap::new()),
            // entity_type_last_run: RwLock::new(HashMap::new()),
            execute_every: 1,
        }
    }

    pub async fn execute(&self) {
        loop {
            // Execute each worker function, passing them the engine
            workers::scrape(&self).await;
            sleep(Duration::from_secs(self.execute_every as u64)).await;
        }
    }

    pub async fn add_node(&self, payload: Payload) -> NodeId {
        // Get new ID after incrementing existing node ID
        let label = payload.to_string();
        let id = {
            let mut id = self.last_node_id.write().await;
            *id += 1;
            *id
        };
        // Store the label in the engine
        {
            let mut labels = self.labels.write().await;
            labels.insert(label.clone());
        };
        // Store the node in the engine
        {
            let mut nodes = self.nodes.write().await;
            nodes.push(Node {
                id,
                label: label.clone(),
                payload,
                written_at: Utc::now(),
            });
        }
        // Store the node in nodes_by_label_id
        {
            let mut nodes_by_label = self.nodes_by_label.write().await;
            nodes_by_label
                .entry(label)
                .and_modify(|entries| entries.push(id))
                .or_insert(vec![id]);
        }
        id
    }

    // pub async fn get_nodes_by_label(&self, label: &str) -> Vec<&Node> {

    // }
}
