impl Engine {
    pub async fn get_node_indexes_of_type(&self, label_id: LabelId) -> Vec<NodeId> {
        let nodes_by_label_id = self.nodes_by_label_id.read().await;
        // If entry for entity_type exists, then we return a flat list of node indexes
        // else we return an empty list
        nodes_by_label_id.get(&label_id).unwrap_or(vec![])
    }

    pub async fn get_node_indexes_of_type_since(
        &self,
        entity_type: LabelId,
        since: &DateTime<Utc>,
    ) -> Vec<NodeId> {
        let features = self.nodes_by_label_id.read().await;
        // If entry for entity_type exists, we return a flat list of node indexes
        // that were written since else we return a list of all node indexes of entity_type
        features.get(&entity_type).map_or(vec![], |chunks| {
            chunks
                .iter()
                .filter(|chunk| chunk.written_at > *since)
                .flat_map(|chunk| chunk.node_indices.clone())
                .collect::<Vec<_>>()
        })
    }

    pub async fn update_feature_nodes_to_graph(&self, entity_type: &LabelId, written: Vec<NodeId>) {
        // Check if entity_type is in app_state.entity_nodes
        // If exists, extend new nodes, else insert new nodes
        let mut entity_nodes = self.nodes_by_label_id.write().await;
        match entity_nodes.get_mut(&entity_type) {
            Some(entry) => {
                entry.push(EntityTypeNodesChunk {
                    written_at: Utc::now(),
                    entity_type: &entity_type,
                    node_indices: written,
                });
            }
            None => {
                entity_nodes.insert(
                    entity_type.clone(),
                    vec![EntityTypeNodesChunk {
                        written_at: Utc::now(),
                        entity_type: entity_type.clone(),
                        node_indices: written,
                    }],
                );
            }
        };
    }

    pub async fn update_last_run(&self, entity_type: &LabelId) {
        let mut last_run = self.entity_type_last_run.write().await;
        last_run.insert(entity_type.clone(), Utc::now());
    }

    pub fn execute_feature(&self, entity_type: &EntityType) {
        let written = {
            let last_run = self.entity_type_last_run.read().unwrap();
            match entity_type {
                EntityType::EmailAccount => {
                    add_email_accounts_to_graph(self, last_run.get(&entity_type))
                }
                EntityType::Mailbox => add_mailboxes_to_graph(self, last_run.get(&entity_type)),
                EntityType::Email => add_emails_to_graph(self, last_run.get(&entity_type)),
                _ => vec![],
            }
        };
        self.update_feature_nodes_to_graph(entity_type, written);
        self.update_last_run(entity_type);
        info!("Updated nodes for entity type {}", entity_type);
    }
}
