// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

/*!
The Email entity represents emails that we use in our daily lives.
An email consists of a sender, a receiver, date, subject, body, etc.
*/

use super::EntityType;
use crate::provider::EntityExtraction;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mailbox {
    pub label: String,
    pub slug: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Email {
    pub id: u32,

    pub from_name: String,
    pub from_email: String,

    // Contacts are processed after saving emails and it depends on certain logic
    // #[ts(type = "number")]
    // pub from_contact_id: Option<i64>,
    pub date: i64,
    pub subject: String,
    pub body_text: String,

    // This is from email headers
    pub message_id: Option<String>,

    // This is from email headers
    pub in_reply_to: Vec<String>,
}

impl EntityExtraction for Email {
    // fn get_payload_content_type(&self) -> String {
    //     "email".to_string()
    // }

    fn get_labels(&self) -> Vec<String> {
        vec![
            EntityType::Person,
            EntityType::Event,
            EntityType::Place,
            EntityType::Date,
            EntityType::SocialGroup,
            EntityType::Organization,
            EntityType::Workplace,
            EntityType::Financial,
            EntityType::Shopping,
            EntityType::News,
            EntityType::NeedHelp,
            EntityType::Question,
            EntityType::Request,
            EntityType::Link,
        ]
        .iter()
        .map(|x| x.to_string())
        .collect()
    }

    // fn get_example_text(&self) -> String {
    //     "I came across Alex Ohom at a Microsoft event here in London.
    //     He wanted to know about the new product I am working on. How are things on your side?
    //     I would love to have a chat with you. Oh btw, I am still looking for a new job."
    //         .to_string()
    // }

    // fn get_example_extractions(&self) -> Vec<(EntityType, String)> {
    //     vec![
    //         (EntityType::Person, "Alex Ohom".to_string()),
    //         (EntityType::Organization, "Microsoft event".to_string()),
    //         (EntityType::Place, "London".to_string()),
    //         (
    //             EntityType::Workplace,
    //             "new product I am working on".to_string(),
    //         ),
    //         (
    //             EntityType::Question,
    //             "How are things on your side?".to_string(),
    //         ),
    //         (EntityType::Request, "would love to have a chat".to_string()),
    //         (EntityType::NeedHelp, "looking for a new job".to_string()),
    //     ]
    // }

    fn get_payload(&self) -> String {
        format!(
            "Email content:
        Subject - {}
        Body - {}
        ",
            self.subject, self.body_text
        )
    }
}

// pub fn add_email_accounts_to_graph(
//     app_state: &PiState,
//     _last_run: Option<&DateTime<Utc>>,
// ) -> Vec<(EntityType, NodeIndex)> {
//     // We insert the email account into our graph
//     let mut graph = app_state.graph.write().unwrap();
//     let email_account_nx = graph.add_node(PiNode::EmailAccount(app_state.email_account.clone()));
//     vec![(EntityType::EmailAccount, email_account_nx)]
// }

// pub fn add_mailboxes_to_graph(
//     app_state: &PiState,
//     last_run: Option<&DateTime<Utc>>,
// ) -> Vec<(EntityType, NodeIndex)> {
//     // We insert all the mailboxes in this email account into our graph
//     let mut written: Vec<(EntityType, NodeIndex)> = vec![];
//     let mut graph = app_state.graph.write().unwrap();
//     let iterator = match last_run {
//         Some(last_run) => {
//             app_state.get_node_indexes_of_type_since(EntityType::EmailAccount, last_run)
//         }
//         None => app_state.get_node_indexes_of_type(EntityType::EmailAccount),
//     };
//     for (email_account_nx) in iterator {
//         let mailboxes = get_mailboxes(&app_state.storage_root, &email_account);
//         for mailbox in mailboxes {
//             let slug = mailbox.slug.clone();
//             let mailbox_nx = graph.add_node(PiNode::Mailbox(mailbox));
//             graph.add_edge(email_account_nx, mailbox_nx, PiEdge::ContainedIn);
//             written.push((slug, mailbox_nx));
//         }
//     }
//     written
// }

// pub fn add_emails_to_graph(
//     app_state: &PiState,
//     last_run: Option<&DateTime<Utc>>,
// ) -> Vec<(EntityType, NodeIndex)> {
//     let written: Vec<(EntityType, NodeIndex)> = vec![];
//     let iterator = match last_run {
//         Some(last_run) => {
//             app_state.get_node_indexes_of_type_since(EntityType::EmailAccount, last_run)
//         }
//         None => app_state.get_node_indexes_of_type(EntityType::EmailAccount),
//     };
//     for email_account_nx in iterator {
//         let emails = read_emails(&app_state.storage_root, &email_account);
//         // We track all the email addresses and their corresponding index in the graph
//         let mut email_address_indices = HashMap::<String, NodeIndex>::new();

//         // We get all the mailbox nodes directly connected to this email account
//         let mut graph = app_state.graph.write().unwrap();

//         // We insert them emails into our graph
//         for email in emails {
//             let email_nx = graph.add_node(PiNode::Email(email.clone()));
//             let mut walker = graph.neighbors(email_account_nx).detach();
//             while let Some(nx) = walker.next_node(&mut graph) {
//                 match &graph[nx] {
//                     PiNode::Mailbox(mailbox) => {
//                         if mailbox.slug == email.mailbox_slug {
//                             // We insert an edge between this email and the mailbox it belongs to
//                             graph.add_edge(nx, email_nx, PiEdge::ContainedIn);
//                             break;
//                         }
//                     }
//                     _ => {}
//                 }
//             }
//             // We insert the contact into our graph if it doesn't exist
//             if email_address_indices.contains_key(&email.from_email) {
//                 graph.add_edge(
//                     email_address_indices[&email.from_email],
//                     email_nx,
//                     PiEdge::Sender,
//                 );
//             } else {
//                 let contact_nx = graph.add_node(PiNode::Contact(Contact {
//                     email_address_list: vec![email.from_email.clone()],
//                     last_email_at: email.date,
//                     ..Default::default()
//                 }));
//                 graph.add_edge(contact_nx, email_nx, PiEdge::Sender);
//                 email_address_indices.insert(email.from_email, contact_nx);
//             }
//         }
//     }
//     written
// }

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub summary: String,
}

#[derive(Debug, Deserialize)]
pub struct Entity {
    pub entity_type: String,
    pub matching_text: String,
}
