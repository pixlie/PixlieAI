// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::{
    engine::{Engine, Node, NodeId, Payload},
    entity::web::{Link, WebPage},
};
use log::info;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use std::any::Any;
pub mod web;

pub trait NodeWorker {
    fn process(&mut self, engine: &Engine, node_id: &NodeId);
}

impl Engine {
    pub fn process_nodes(&self) {
        info!("Processing nodes");
        self.nodes.write().unwrap().par_iter_mut().for_each(|node| {
            // We get the node from the engine
            match node.payload {
                Payload::Link(ref mut payload) => {
                    payload.process(self, &node.id);
                }
                Payload::FileHTML(ref mut payload) => {
                    payload.process(self, &node.id);
                }
                _ => {}
            }
        });
    }
}
