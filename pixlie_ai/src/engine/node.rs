// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use crate::engine::{Engine, NodeFlags};
use crate::entity::classifier::{Classification, Classifier, ClassifierSettings};
use crate::entity::content::TableRow;
use crate::entity::crawler::CrawlerSettings;
use crate::entity::named_entity::{EntityExtraction, EntityName, ExtractedEntity};
use crate::entity::objective::Objective;
use crate::entity::project_settings::ProjectSettings;
use crate::entity::search::web_search::WebSearch;
use crate::entity::web::domain::Domain;
use crate::entity::web::link::Link;
use crate::entity::web::web_metadata::WebMetadata;
use crate::entity::web::web_page::WebPage;
use crate::error::PiResult;
use crate::{ExternalData, FetchError, FetchRequest, FetchResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::cmp::Ordering;
use std::sync::Arc;
use strum::{Display, EnumString};
use ts_rs::TS;
use utoipa::ToSchema;

pub trait Node: Send + Sync {
    fn process(&self, node_id: NodeId, engine: Arc<&Engine>) -> PiResult<()>;
    fn as_any(&self) -> &dyn Any;
    fn get_labels(&self) -> Vec<NodeLabel>;
}

pub trait Fetchable: Node {
    fn make_fetch_requests(&self, node_id: NodeId, engine: Arc<&Engine>) -> PiResult<Vec<FetchRequest>>;
    fn handle_fetch_response(&self, node_id: NodeId, engine: Arc<&Engine>, response: FetchResponse) -> PiResult<()>;
    fn handle_fetch_error(&self, node_id: NodeId, engine: Arc<&Engine>, error: FetchError) -> PiResult<()>;
}

#[derive(Clone, Display, Deserialize, Serialize)]
pub enum Payload {
    Link(Link),
    WebMetadata(WebMetadata),
    Text(String),
    Tree, // Tree can contain nodes of any payload type, including other trees
    TableRow(TableRow),
    ProjectSettings(ProjectSettings),
    CrawlerSettings(CrawlerSettings),
    ClassifierSettings(ClassifierSettings),
    Classification(Classification),
    NamedEntitiesToExtract(Vec<EntityName>),
    ExtractedNamedEntities(Vec<ExtractedEntity>),
}

pub(crate) type NodeId = u32;

pub(crate) type ArcedNodeId = Arc<NodeId>;

#[derive(
    Clone,
    Debug,
    Deserialize,
    Display,
    EnumString,
    Eq,
    Hash,
    Ord,
    PartialOrd,
    PartialEq,
    Serialize,
    ToSchema,
    TS,
)]
#[ts(export)]
pub enum NodeLabel {
    AddedByUser,
    AddedByAI,
    AddedByWebSearch,
    AddedByGliner,

    Objective,

    DomainName,
    Link,
    RobotsTxt,

    Content,
    Heading,
    Title,
    Paragraph,
    ListItem,
    UnorderedPoints,
    OrderedPoints,
    Partial,
    SearchTerm,

    WebPage,
    WebSearch,
    WebMetadata,
    CrawlCondition,
    ProjectSettings,
    CrawlerSettings,
    ClassifierSettings,
    Classification,
    NamedEntitiesToExtract,
    ExtractedNamedEntities,
}

impl Default for NodeFlags {
    fn default() -> Self {
        NodeFlags::empty()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeItem {
    pub id: NodeId,
    pub labels: Vec<NodeLabel>, // A node can have multiple labels, like tags
    pub payload: Payload,

    pub flags: NodeFlags,
    pub written_at: DateTime<Utc>,
}

pub struct TypedNodeItem<T: Node> {
    pub id: NodeId,
    pub node: T,
    pub flags: NodeFlags,
    pub written_at: DateTime<Utc>,
}

impl<T: Node> TypedNodeItem<T> {
    pub fn new(id: NodeId, node: T) -> Self {
        Self {
            id,
            node,
            flags: NodeFlags::empty(),
            written_at: Utc::now(),
        }
    }

    pub fn process(&self, engine: Arc<&Engine>) -> PiResult<()> {
        self.node.process(self.id, engine)
    }

    pub fn make_fetch_requests(&self, engine: Arc<&Engine>) -> PiResult<Vec<FetchRequest>> 
    where 
        T: Fetchable 
    {
        self.node.make_fetch_requests(self.id, engine)
    }

    pub fn handle_fetch_response(&self, engine: Arc<&Engine>, response: FetchResponse) -> PiResult<()> 
    where 
        T: Fetchable 
    {
        self.node.handle_fetch_response(self.id, engine, response)
    }

    pub fn handle_fetch_error(&self, engine: Arc<&Engine>, error: FetchError) -> PiResult<()> 
    where 
        T: Fetchable 
    {
        self.node.handle_fetch_error(self.id, engine, error)
    }

    pub fn get_labels(&self) -> Vec<NodeLabel> {
        self.node.get_labels()
    }

    pub fn get_node(&self) -> &T {
        &self.node
    }

    pub fn get_node_mut(&mut self) -> &mut T {
        &mut self.node
    }
}

pub enum AnyTypedNode {
    Domain(TypedNodeItem<crate::entity::web::domain::Domain>),
    // Link(TypedNodeItem<crate::entity::web::link::Link>),
    // WebPage(TypedNodeItem<crate::entity::web::web_page::WebPage>),
    // Add more as you implement them
}

impl AnyTypedNode {
    pub fn process(&self, engine: Arc<&Engine>) -> PiResult<()> {
        match self {
            AnyTypedNode::Domain(node) => node.process(engine),
            // AnyTypedNode::Link(node) => node.process(engine),
            // AnyTypedNode::WebPage(node) => node.process(engine),
        }
    }

    pub fn handle_fetch_response(&self, engine: Arc<&Engine>, response: FetchResponse) -> PiResult<()> {
        match self {
            AnyTypedNode::Domain(node) => node.handle_fetch_response(engine, response),
        }
    }

    pub fn handle_fetch_error(&self, engine: Arc<&Engine>, error: FetchError) -> PiResult<()> {
        match self {
            AnyTypedNode::Domain(node) => node.handle_fetch_error(engine, error),
        }
    }

    pub fn get_labels(&self) -> Vec<NodeLabel> {
        match self {
            AnyTypedNode::Domain(node) => node.get_labels(),
        }
    }

    pub fn get_id(&self) -> NodeId {
        match self {
            AnyTypedNode::Domain(node) => node.id,
        }
    }
}

pub fn create_typed_node(id: NodeId, labels: &[NodeLabel], payload: &Payload) -> Option<AnyTypedNode> {
    use crate::entity::web::domain::Domain;
    
    if labels.contains(&NodeLabel::DomainName) {
        if let Payload::Text(domain_name) = payload {
            return Some(AnyTypedNode::Domain(TypedNodeItem::new(id, Domain::new(domain_name.clone()))));
        }
    }
    
    // Add more entity types here as you implement them
    // if labels.contains(&NodeLabel::Link) { ... }
    // if labels.contains(&NodeLabel::WebPage) { ... }
    
    None
}

impl NodeItem {
    pub(super) fn process(&self, arced_engine: Arc<&Engine>) -> PiResult<()> {
        if self.labels.contains(&NodeLabel::DomainName) {
            Domain::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::Link) {
            Link::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::Objective) {
            Objective::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::WebSearch) {
            WebSearch::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::WebPage) {
            WebPage::process(self, arced_engine.clone(), None)?;
            Classifier::process(self, arced_engine.clone(), None)?;
            EntityExtraction::process(self, arced_engine.clone(), None)?;
        }
        Ok(())
    }

    pub(super) fn handle_fetch_response(
        &self,
        arced_engine: Arc<&Engine>,
        response: FetchResponse,
    ) -> PiResult<()> {
        if self.labels.contains(&NodeLabel::DomainName) {
            Domain::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Response(response)),
            )?;
        } else if self.labels.contains(&NodeLabel::Link) {
            Link::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Response(response)),
            )?;
        } else if self.labels.contains(&NodeLabel::Objective) {
            Objective::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Response(response)),
            )?;
        } else if self.labels.contains(&NodeLabel::WebSearch) {
            WebSearch::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Response(response)),
            )?;
        } else if self.labels.contains(&NodeLabel::WebPage) {
            WebPage::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Response(response.clone())),
            )?;
            Classifier::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Response(response.clone())),
            )?;
            EntityExtraction::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Response(response.clone())),
            )?;
        }
        Ok(())
    }

    pub(super) fn handle_fetch_error(
        &self,
        arced_engine: Arc<&Engine>,
        error: FetchError,
    ) -> PiResult<()> {
        if self.labels.contains(&NodeLabel::DomainName) {
            Domain::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        } else if self.labels.contains(&NodeLabel::Link) {
            Link::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        } else if self.labels.contains(&NodeLabel::Objective) {
            Objective::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        } else if self.labels.contains(&NodeLabel::WebSearch) {
            WebSearch::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        } else if self.labels.contains(&NodeLabel::WebPage) {
            WebPage::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Error(error.clone())),
            )?;
            Classifier::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Error(error.clone())),
            )?;
            EntityExtraction::process(
                self,
                arced_engine.clone(),
                Some(ExternalData::Error(error.clone())),
            )?;
        }
        Ok(())
    }
}

impl Ord for NodeItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for NodeItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for NodeItem {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for NodeItem {}

pub type ArcedNodeItem = Arc<NodeItem>;

pub enum ExistingOrNewNodeId {
    Existing(NodeId),
    New(NodeId),
}

impl ExistingOrNewNodeId {
    pub fn get_node_id(&self) -> NodeId {
        match self {
            ExistingOrNewNodeId::Existing(id) => id.clone(),
            ExistingOrNewNodeId::New(id) => id.clone(),
        }
    }
}
