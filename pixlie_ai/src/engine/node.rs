use crate::engine::{Engine, NodeFlags};
use crate::entity::content::TableRow;
use crate::entity::objective::Objective;
use crate::entity::search::web_search::WebSearch;
use crate::entity::web::domain::Domain;
use crate::entity::web::link::Link;
use crate::entity::web::web_page::WebPage;
use crate::entity::workflow::WorkflowStep;
use crate::error::PiResult;
use crate::{ExternalData, FetchError, FetchResponse};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::sync::Arc;
use strum::{Display, EnumString};
use ts_rs::TS;

#[derive(Clone, Display, Deserialize, Serialize)]
pub enum Payload {
    // StepPrompt(String),
    Step(WorkflowStep),
    Link(Link),
    Text(String),
    Tree, // Tree can contain nodes of any payload type, including other trees
    TableRow(TableRow),
}

pub(crate) type NodeId = u32;
pub(crate) type ArcedNodeId = Arc<NodeId>;

#[derive(Clone, Debug, Deserialize, Display, EnumString, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
pub enum NodeLabel {
    AddedByUser,
    AddedByAI,
    AddedByWebSearch,
    Content,
    Domain,
    Heading,
    Link,
    ListItem,
    Objective,
    OrderedPoints,
    Paragraph,
    Partial,
    RobotsTxt,
    SearchTerm,
    Title,
    UnorderedPoints,
    WebPage,
    WebSearch,
    CrawlCondition,
}

impl Default for NodeFlags {
    fn default() -> Self {
        NodeFlags::empty()
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct NodeItem {
    pub id: NodeId,
    pub labels: Vec<NodeLabel>, // A node can have multiple labels, like tags, indexed by relevance
    pub payload: Payload,

    pub flags: NodeFlags,
    pub written_at: DateTime<Utc>,
}

impl NodeItem {
    pub(super) fn process(&self, arced_engine: Arc<&Engine>) -> PiResult<()> {
        if self.labels.contains(&NodeLabel::Domain) {
            Domain::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::Link) {
            Link::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::WebPage) {
            WebPage::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::Objective) {
            Objective::process(self, arced_engine.clone(), None)?;
        } else if self.labels.contains(&NodeLabel::WebSearch) {
            WebSearch::process(self, arced_engine.clone(), None)?;
        }
        Ok(())
    }

    pub(super) fn handle_fetch_response(
        &self,
        arced_engine: Arc<&Engine>,
        response: FetchResponse,
    ) -> PiResult<()> {
        if self.labels.contains(&NodeLabel::Domain) {
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
        }
        Ok(())
    }

    pub(super) fn handle_fetch_error(
        &self,
        arced_engine: Arc<&Engine>,
        error: FetchError,
    ) -> PiResult<()> {
        if self.labels.contains(&NodeLabel::Domain) {
            Domain::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        } else if self.labels.contains(&NodeLabel::Link) {
            Link::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        } else if self.labels.contains(&NodeLabel::Objective) {
            Objective::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        } else if self.labels.contains(&NodeLabel::WebSearch) {
            WebSearch::process(self, arced_engine.clone(), Some(ExternalData::Error(error)))?;
        }
        Ok(())
    }
}

impl NodeItem {
    pub fn get_label(&self) -> String {
        self.payload.to_string()
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
