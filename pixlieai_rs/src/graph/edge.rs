// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

pub enum PiEdge {
    ContainedIn,
    Sender,
    Receiver,

    // The following edges connect multiple nodes of the graph to a Bucket node.
    Similar,      // Nodes have similar content
    Thread,       // Nodes are in a thread
    Topic,        // Nodes belong to a topic
    Organization, // Nodes belong to a organization
    Role,         // Nodes belong to a role of the user
    TimePeriod,   // Nodes belong to a time period
}
