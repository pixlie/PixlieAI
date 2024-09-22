// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::entity::{
    email::{Email, Mailbox},
    organization::Organization,
    people::{Contact, Person},
    EntityType, ExtractedEntity,
};

pub enum PiNode {
    Event(String),
    Place(String),
    Date(String),
    Financial(String),
    Shopping(String),
    News(String),
    Seek(String),
    Question(String),
    Request(String),
    Content(String),
    Group(String),
    Link(String),
    Title(String),

    EmailAccount(String),
    Mailbox(Mailbox),
    Email(Email),
    Person(Person),
    Contact(Contact),
    Organization(Organization),
}

// pub fn save_entity_to_graph(
//     app_state: AppState,
//     current_node: PiNode,
//     extracted_entity: ExtractedEntity,
// ) {
//     match entity.entity_type {
//         EntityType::Event => PiNode::Event(entity.matching_text),
//         EntityType::Place => PiNode::Place(entity.matching_text),
//         EntityType::Date => PiNode::Date(entity.matching_text),
//         EntityType::Financial => PiNode::Financial(entity.matching_text),
//         EntityType::Shopping => PiNode::Shopping(entity.matching_text),
//         EntityType::News => PiNode::News(entity.matching_text),
//         EntityType::NeedHelp => PiNode::Seek(entity.matching_text),
//         EntityType::Question => PiNode::Question(entity.matching_text),
//         EntityType::Request => PiNode::Request(entity.matching_text),
//         EntityType::Link => PiNode::Link(entity.matching_text),

//         EntityType::Organization => PiNode::Organization(Organization {
//             id: None,
//             name: entity.matching_text,
//         }),
//     }
// }
