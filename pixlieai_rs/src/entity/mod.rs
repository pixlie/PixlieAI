// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

/*!
Here we define all the entities that we can extract from the data. Each entity is
stored in the graph as a node and edges are created to represent the relationships
between them.

Not all entity types have a corresponding node. Some nodes have an internal type
and therefore many entities may point to the same node.
*/

use strum::{Display, EnumString};

pub mod content;
pub mod email;
pub mod organization;
pub mod people;

#[derive(Clone, EnumString, Display, Hash, PartialEq, Eq)]
pub enum EntityType {
    Date,
    Event,
    Person,
    Place,
    SocialGroup,
    Organization,
    Workplace,
    Financial,
    Shopping,
    News,
    NeedHelp,
    Question,
    Request,

    Title,
    EmailAccount,
    Mailbox,
    Email,
    Link,
}

pub struct ExtractedEntity {
    pub entity_type: EntityType,
    pub matching_text: String,
}
