// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

use crate::{config::Settings, engine::Engine};
use chrono::{TimeDelta, Utc};
use log::info;
use petgraph::{graph::node_index, visit::Dfs, Directed, Graph};
use std::{collections::HashMap, ops::Deref, sync::RwLock};

pub async fn run_pixlie_engine(settings: Settings, initial_labels: Vec<String>) {
    let engine = Engine::new(initial_labels);
    info!("Initialized engine");

    // We find the top 10 contacts with the most emails
    let mut counted_contacts: Vec<(String, usize)> = vec![];
    let mut visited_contacts = 0;
    let mut visited_nodes = 0;
    let mut email_subjects: Vec<String> = vec![];
    let today = Utc::now();

    let graph = app_state.graph.read().unwrap();
    let mut dfs = Dfs::new(graph.deref(), node_index(0));
    while let Some(nx) = dfs.next(graph.deref()) {
        visited_nodes += 1;
        match graph[nx] {
            PiNode::EmailAccount(ref email_account) => {
                println!("Email account node {}", email_account);
            }
            PiNode::Mailbox(_) => {}
            PiNode::Contact(ref contact) => {
                // Ignore contacts which do not have emails within the last 12 months
                if contact.last_email_at
                    < today
                        .checked_sub_signed(TimeDelta::weeks(52))
                        .unwrap()
                        .timestamp()
                {
                    continue;
                }

                visited_contacts += 1;
                counted_contacts.push((
                    contact.email_address_list.join(", "),
                    graph.edges(nx).count(),
                ));
            }
            PiNode::Email(ref email) => {
                // If email is older than 1 week, ignore it
                if email.date
                    < today
                        .checked_sub_signed(TimeDelta::weeks(4))
                        .unwrap()
                        .timestamp()
                {
                    continue;
                }

                if email.subject.contains("unsubscribe") {
                    continue;
                }

                if email_subjects.len() == 0 {
                    // Insert the first subject
                    email_subjects.push(email.subject.trim().to_string());
                } else {
                    let mut is_similar = false;
                    for subject in email_subjects.iter() {
                        if find_sentence_similarity(&email.subject, &subject) > 0.7 {
                            // We found a match
                            is_similar = true;
                            break;
                        }
                    }
                    match email_subjects.binary_search(&email.subject) {
                        Ok(_) => {}
                        Err(index) => {
                            if !is_similar {
                                email_subjects.insert(index, email.subject.clone());
                                info!(
                                    "\n\n************\nExtracting entities from email {}\n",
                                    email.subject
                                );
                                extract_entities_from_email_with_llm(email);
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    // We print the top 10 after ordering the HashMap
    counted_contacts.sort_by(|a, b| b.1.cmp(&a.1));
    info!("Visited contacts: {}", visited_contacts);
    info!("Visited nodes: {}", visited_nodes);
    info!(
        "Top 10 contacts: {:?}",
        counted_contacts.iter().take(10).collect::<Vec<_>>()
    );
}
