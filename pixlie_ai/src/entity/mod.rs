// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

/*!
Here we define all the entities that we can extract from the data. Each entity is
stored in the graph as a node, and edges are created to represent the relationships
between them.

Not all entity types have a corresponding node. Some nodes have an internal type,
and therefore many entities may point to the same node.
*/

pub mod classifier;
pub mod content;
pub mod crawler;
pub mod email;
pub mod named_entity;
pub mod objective;
pub mod pixlie;
pub mod project_settings;
pub mod search;
pub mod text;
pub mod web;
