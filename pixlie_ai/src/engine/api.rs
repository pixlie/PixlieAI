// Copyright 2025 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/pixlie/PixlieAI/blob/main/LICENSE

use super::node::{ArcedNodeItem, NodeLabel};
use super::{EdgeLabel, Engine, NodeFlags};
use crate::engine::node::{NodeId, NodeItem, Payload};
use crate::entity::classifier::{Classification, ClassifierSettings};
use crate::entity::content::TableRow;
use crate::entity::crawler::CrawlerSettings;
use crate::entity::named_entity::{EntityName, ExtractedEntity};
use crate::entity::project_settings::ProjectSettings;
use crate::entity::search::saved_search::SavedSearch;
use crate::entity::web::domain::{Domain, FindDomainOf};
use crate::entity::web::link::Link;
use crate::entity::web::web_metadata::WebMetadata;
use crate::error::PiError;
use crate::PiEvent;
use crate::{api::ApiState, error::PiResult};
use actix_web::{get, post, web, Responder};
use itertools::Itertools;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;
use strum::Display;
use ts_rs::TS;
use utoipa::{IntoParams, ToSchema};

#[derive(Clone)]
pub struct EngineRequest {
    pub request_id: u32,
    pub project_id: String,
    pub payload: EngineRequestPayload,
}

#[derive(Clone, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct LinkWrite {
    /// The URL of the link
    #[schema(value_type = url::Url)]
    pub url: String,
}

#[derive(Clone, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct ProjectSettingsWrite {
    pub extract_data_only_from_specified_links: bool,
    pub crawl_within_domains_of_specified_links: bool,
    pub crawl_direct_links_from_specified_links: bool,
}

#[derive(Clone, Deserialize, Display, ToSchema, TS)]
#[ts(export)]
pub enum NodeWrite {
    Link(LinkWrite),
    SearchTerm(String),
    Objective(String),
    ProjectSettings(ProjectSettingsWrite),
}

#[derive(Clone, Deserialize, ToSchema, TS)]
#[ts(export)]
pub struct EdgeWrite {
    node_ids: (NodeId, NodeId),
    edge_labels: (EdgeLabel, EdgeLabel),
}

#[derive(Clone, Deserialize, TS)]
#[ts(export)]
pub enum EngineRequestPayload {
    Explore(Option<u32>), // Optional node id to start from

    GetLabels,
    GetEntities,
    GetClassifications,
    GetNodesWithLabel(String),
    GetNodesWithIds(Vec<u32>),
    GetAllNodes(i64),
    GetAllEdges(i64),

    CreateNode(NodeWrite),
    CreateEdge(EdgeWrite),

    // Some nodes allow a "query", which can generate any number of nodes, like a search
    Query(u32),
}

/// A list of all outgoing edges of a node, with the ID of the node and the label of the edge.
/// The UNIX timestamp represents when a node's edge list was last written to.
#[derive(Clone, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct APINodeEdges {
    // TODO: The edges should be Vec<(NodeId, EdgeLabel)>
    // Change this and handle chain-effects, if any
    #[ts(type = "Array<[number, string]>")]
    pub edges: Vec<(NodeId, String)>,
    pub written_at: i64,
}

/// A map of node IDs to their outgoing edges.
#[derive(Clone, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct APIEdges(HashMap<NodeId, APINodeEdges>);

/// Schema for response to the `explore` API request.
#[derive(Clone, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct Explore {
    pub nodes: Vec<APINodeItem>,
    pub edges: APIEdges,
    pub sibling_nodes: Vec<Vec<NodeId>>, // Nodes that are grouped together because they are siblings
}

#[derive(Clone, Serialize, TS, ToSchema)]
pub struct EntityGroup {
    pub entity_name: String,
    pub extracted_text: Vec<String>,
}

#[derive(Clone, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct ClassifiedItem {
    pub url: String,
    pub is_relevant: bool,
    pub reason: String,
    pub insight: Option<String>,
}

#[derive(Clone, Serialize, TS, ToSchema)]
#[ts(export)]
pub struct Matches {
    pub urls: Vec<String>,
    pub web_pages: Vec<WebPageMatch>,
}

/// Engine's response for an API request.
///
/// API requests for a project are sent to its engine.
/// The engine responds with an `EngineResponsePayload`, which is directly passed on as
/// the API response.
#[derive(Clone, Serialize, ToSchema, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum EngineResponsePayload {
    /// Response for node creation. Returns the ID of the created node.
    NodeCreatedSuccessfully(NodeId),
    /// Response for edge creation.
    EdgeCreatedSuccessfully,
    /// Response for a node query. Returns a list of nodes.
    Nodes(Vec<APINodeItem>),
    /// Response for edge retrieval. Returns a list of edges.
    Edges(APIEdges),
    // TODO: The below should be Labels(Vec<NodeLabel>)
    // Change this and handle chain-effects, if any
    /// Response for label retrieval. Returns a list of labels.
    Labels(Vec<String>),
    /// Response for entities retrieval. Returns a list of entities.
    Entities(Vec<EntityGroup>),
    /// Response for classifications retrieval. Returns a list of classifications.
    Classifications(Vec<ClassifiedItem>),
    Explore(Explore),
    /// Error response.
    Error(String),
}

/// All nodes contain a data payload.
/// APIPayload is the schema for this payload and contains the type of payload and the data.
#[derive(Clone, Display, Deserialize, Serialize, ToSchema, TS)]
#[serde(tag = "type", content = "data")]
#[ts(export)]
pub enum APIPayload {
    /// A relative link to a resource, with the containing node connected to it's owner.
    /// Currently only used for nodes representing web URLs, in which case the owner is
    /// a node representing its domain.
    Link(Link),
    /// A metadata payload for a web page.
    WebMetadata(WebMetadata),
    /// A text payload.
    Text(String),
    /// [WIP] A tree payload.
    Tree(String),
    /// [WIP] A table row payload.
    TableRow(TableRow),
    /// These are the settings of a project, based on which Pixlie AI
    /// operates in the context of the project.
    ProjectSettings(ProjectSettings),
    /// These are the settings of the Pixlie AI crawler, based on which it
    /// crawls the web.
    CrawlerSettings(CrawlerSettings),
    /// These are the settings of the Pixlie AI classifier, based on which it classifies content.
    ClassifierSettings(ClassifierSettings),
    /// This stores how a content was classified by LLM.
    Classification(Classification),
    /// These are named entities that we should extract from the content if the content is classified as relevant.
    NamedEntitiesToExtract(Vec<EntityName>),
    /// These are the extracted named entities from the content if the content is classified as relevant.
    ExtractedNamedEntities(Vec<ExtractedEntity>),
}

#[derive(Clone, Default, Serialize, ToSchema, TS)]
#[ts(export)]
pub enum APINodeFlags {
    #[default]
    None, // Not used
    IsProcessed,
    IsRequesting,
    IsBlocked,
    HadError,
}

impl APINodeFlags {
    pub fn from_node_flags(flags: &NodeFlags) -> Vec<APINodeFlags> {
        let mut api_flags = vec![];
        if flags.contains(NodeFlags::IS_PROCESSED) {
            api_flags.push(APINodeFlags::IsProcessed);
        }
        if flags.contains(NodeFlags::IS_REQUESTING) {
            api_flags.push(APINodeFlags::IsRequesting);
        }
        if flags.contains(NodeFlags::IS_BLOCKED) {
            api_flags.push(APINodeFlags::IsBlocked);
        }
        if flags.contains(NodeFlags::HAD_ERROR) {
            api_flags.push(APINodeFlags::HadError);
        }
        api_flags
    }
}

/// Schema for a node in any API response.
#[derive(Clone, Serialize, ToSchema, TS)]
#[ts(export)]
pub struct APINodeItem {
    /// The ID of the node
    pub id: NodeId,
    /// The labels of the node. A node can have multiple labels, like tags, indexed by relevance.
    pub labels: Vec<NodeLabel>,
    /// The payload of the node. This can be a link, text, or a tree.
    pub payload: APIPayload,

    /// The flags of the node. This can be used to indicate if the node is processed, requesting, etc.
    pub flags: Vec<APINodeFlags>,
    /// Unix timestamp of when the node was last written to.
    pub written_at: i64,
}

impl APINodeItem {
    pub fn from_node(arced_node: &ArcedNodeItem) -> APINodeItem {
        let payload: APIPayload = match &arced_node.payload {
            Payload::Text(text) => {
                if arced_node.labels.contains(&NodeLabel::WebPage)
                    || arced_node.labels.contains(&NodeLabel::RobotsTxt)
                {
                    APIPayload::Text("".to_string())
                } else {
                    APIPayload::Text(text.to_string())
                }
            }
            Payload::Link(link) => APIPayload::Link(link.clone()),
            Payload::WebMetadata(web_metadata) => APIPayload::WebMetadata(web_metadata.clone()),
            // The empty string is garbage, just to keep the type system happy
            Payload::Tree => APIPayload::Tree("".to_string()),
            Payload::TableRow(table_row) => APIPayload::TableRow(table_row.clone()),
            Payload::ProjectSettings(project_settings) => {
                APIPayload::ProjectSettings(project_settings.clone())
            }
            Payload::CrawlerSettings(crawler_settings) => {
                APIPayload::CrawlerSettings(crawler_settings.clone())
            }
            Payload::ClassifierSettings(classifier_settings) => {
                APIPayload::ClassifierSettings(classifier_settings.clone())
            }
            Payload::Classification(classification) => {
                APIPayload::Classification(classification.clone())
            }
            Payload::NamedEntitiesToExtract(named_entities_to_extract) => {
                APIPayload::NamedEntitiesToExtract(named_entities_to_extract.clone())
            }
            Payload::ExtractedNamedEntities(extracted_named_entities) => {
                APIPayload::ExtractedNamedEntities(extracted_named_entities.clone())
            }
        };
        APINodeItem {
            id: arced_node.id,
            labels: arced_node.labels.clone(),
            payload,
            flags: APINodeFlags::from_node_flags(&arced_node.flags),
            written_at: arced_node.written_at.timestamp_millis(),
        }
    }
}

#[derive(Clone)]
pub struct EngineResponse {
    pub request_id: u32,
    pub payload: EngineResponsePayload,
}

#[derive(Deserialize, IntoParams)]
pub struct QueryNodes {
    // TODO: The below should be Option<NodeLabel>
    // Change this and handle chain-effects, if any
    /// The node label to filter nodes by.
    /// If provided, ids & since will be ignored.
    label: Option<String>,
    /// A comma-separated list of node IDs to filter nodes by.
    /// If provided, since will be ignored.
    ids: Option<String>,
    /// The timestamp (in milliseconds) to filter nodes by.
    /// Nodes written after this timestamp will be returned.
    since: Option<i64>,
}

#[derive(Deserialize, IntoParams)]
pub struct QueryEdges {
    /// The timestamp (in milliseconds) to filter edges by.
    /// Edges written after this timestamp will be returned.
    since: Option<i64>,
}

#[derive(Deserialize, IntoParams)]
pub struct QueryClassifications {
    /// The optional `is_relevant` flag to filter classifications.
    /// If `Some(true)`, only relevant classifications are returned.
    /// If `Some(false)`, only irrelevant classifications are returned.
    /// If `None`, classifications of both relevant and irrelevant are returned.
    is_relevant: Option<bool>,
}

#[derive(Deserialize, IntoParams)]
pub struct QueryEntities {
    /// The optional `entity_name` to filter entities by.
    /// If provided, only entities with this name will be returned.
    /// If `None`, all entities will be returned.
    entity_name: Option<String>,
}

#[utoipa::path(
    path = "/engine/{project_id}/explore",
    responses(
        (
            status = 200,
            description = "Explore data retrieved successfully. Returns `EngineResponsePayload` of `type` `Explore` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
    ),
    tag = "engine",
)]
#[get("/explore")]
pub async fn explore(
    project_id: web::Path<String>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to explore",
        request_id, project_id
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::Explore(None),
        },
    ))?;

    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload);
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

/// Get all labels for a project
#[utoipa::path(
    path = "/engine/{project_id}/labels",
    responses(
        (
            status = 200,
            description = "Labels retrieved successfully. Returns `EngineResponsePayload` of `type` `Labels` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
    ),
    tag = "engine",
)]
#[get("/labels")]
pub async fn get_labels(
    project_id: web::Path<String>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to get all labels",
        request_id, project_id
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetLabels,
        },
    ))?;

    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    let x = match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    };
    x
}

/// Get all nodes for a project
#[utoipa::path(
    path = "/engine/{project_id}/nodes",
    responses(
        (
            status = 200,
            description = "Nodes retrieved successfully. Returns `EngineResponsePayload` of `type` `Nodes` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
        QueryNodes,
    ),
    tag = "engine",
)]
#[get("/nodes")]
pub async fn get_nodes(
    project_id: web::Path<String>,
    params: web::Query<QueryNodes>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();
    if let Some(label) = &params.label {
        debug!(
            "API request {} for project {} to get nodes with label {}",
            request_id, project_id, label
        );

        api_state.main_tx.send(PiEvent::APIRequest(
            project_id.clone(),
            EngineRequest {
                request_id: request_id.clone(),
                project_id: project_id.clone(),
                payload: EngineRequestPayload::GetNodesWithLabel(label.clone()),
            },
        ))?;
    } else if let Some(ids) = &params.ids {
        let u32_ids: Vec<u32> = ids
            .split(",")
            .map(|id| id.parse::<u32>().unwrap())
            .collect();
        if u32_ids.len() == 0 {
            return Err(PiError::InternalError(format!(
                "No IDs provided for API request {} for project {}",
                request_id, project_id
            )));
        }
        debug!(
            "API request {} for project {} to get nodes with ids {:?}",
            request_id,
            project_id,
            ids.split(",").collect::<Vec<&str>>()
        );

        api_state.main_tx.send(PiEvent::APIRequest(
            project_id.clone(),
            EngineRequest {
                request_id: request_id.clone(),
                project_id: project_id.clone(),
                payload: EngineRequestPayload::GetNodesWithIds(u32_ids),
            },
        ))?;
    } else {
        let since = if let Some(since) = params.since {
            since
        } else {
            0
        };
        // Read the nodes written since the given timestamp
        debug!(
            "API request {} for project {} to get nodes since {}",
            request_id, project_id, since
        );
        api_state.main_tx.send(PiEvent::APIRequest(
            project_id.clone(),
            EngineRequest {
                request_id: request_id.clone(),
                project_id: project_id.clone(),
                payload: EngineRequestPayload::GetAllNodes(since),
            },
        ))?;
    }

    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

/// Get all edges for a project
#[utoipa::path(
    path = "/engine/{project_id}/edges",
    responses(
        (
            status = 200,
            description = "Edges retrieved successfully. Returns `EngineResponsePayload` of `type` `Edges` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
        QueryEdges,
    ),
    tag = "engine",
)]
#[get("/edges")]
pub async fn get_edges(
    project_id: web::Path<String>,
    params: web::Query<QueryEdges>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to get all edges",
        request_id, project_id
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    let since = if let Some(since) = params.since {
        since
    } else {
        0
    };
    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetAllEdges(since),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

/// Create a new node for a project
#[utoipa::path(
    path = "/engine/{project_id}/nodes",
    request_body = NodeWrite,
    responses(
        (
            status = 200,
            description = "Node created successfully. Returns `EngineResponsePayload` of `type` `NodeCreatedSuccessfully` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
    ),
    tag = "engine",
)]
#[post("/nodes")]
pub async fn create_node(
    project_id: web::Path<String>,
    node: web::Json<NodeWrite>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to create node with label {}",
        request_id,
        project_id,
        node.to_string()
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::CreateNode(node.into_inner()),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

/// Create a new edge for a project
#[utoipa::path(
    path = "/engine/{project_id}/edges",
    request_body = EdgeWrite,
    responses(
        (
            status = 200,
            description = "Edge created successfully. Returns `EngineResponsePayload` of `type` `EdgeCreatedSuccessfully` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
    ),
    tag = "engine",
)]
#[post("/edges")]
pub async fn create_edge(
    project_id: web::Path<String>,
    edge: web::Json<EdgeWrite>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();
    debug!(
        "API request {} for project {} to create edge between {} and {} with labels {} and {}",
        request_id,
        project_id,
        &edge.node_ids.0,
        &edge.node_ids.1,
        &edge.edge_labels.0,
        &edge.edge_labels.1,
    );
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::CreateEdge(edge.into_inner()),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

/// [Incomplete] Get the results of a search for a node in a project
///
/// This endpoint contains outdated implementation and may not work as expected.
#[utoipa::path(
    path = "/engine/{project_id}/query/{node_id}",
    responses(
        (
            status = 200,
            description = "Query results retrieved successfully. Returns `EngineResponsePayload` of `type` `Nodes` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
        (
            "node_id" = NodeId,
            description = "The ID of the node to query",
            example = 123
        ),
    ),
    tag = "engine",
)]
#[get("/query/{node_id}")]
pub async fn search_results(
    path: web::Path<(String, u32)>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let (project_id, node_id) = path.into_inner();
    // Subscribe to the API channel, so we can receive the response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::Query(node_id),
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(response) => Ok(web::Json(response)),
        None => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".to_string(),
        ))),
    }
}

/// Get all entities for a project
#[utoipa::path(
    path = "/engine/{project_id}/entities",
    responses(
        (
            status = 200,
            description = "Entities retrieved successfully. Returns `EngineResponsePayload` of `type` `Entities` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
        QueryEntities,
    ),
    tag = "engine",
)]
#[get("/entities")]
pub async fn get_entities(
    project_id: web::Path<String>,
    params: web::Query<QueryEntities>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();

    debug!(
        "API request {} for project {} to get entities",
        request_id, project_id
    );

    // Subscribe to receive engine response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetEntities,
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(EngineResponsePayload::Entities(entities)) => {
            let filtered_entities = if let Some(entity_name) = &params.entity_name {
                entities
                    .into_iter()
                    .filter(|entity| entity.entity_name == *entity_name)
                    .collect::<Vec<_>>()
            } else {
                entities
            };

            Ok(web::Json(EngineResponsePayload::Entities(
                filtered_entities,
            )))
        }
        _ => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".into(),
        ))),
    }
}

/// Get all classifications for a project
#[utoipa::path(
    path = "/engine/{project_id}/classifications",
    responses(
        (
            status = 200,
            description = "Classifications retrieved successfully. Returns `EngineResponsePayload` of `type` `Classifications` or `Error`.",
            body = EngineResponsePayload
        ),
        (status = 500, description = "Internal server error"),
    ),
    params(
        (
            "project_id" = uuid::Uuid,
            description = "The ID of the project",
            example = "123e4567-e89b-12d3-a456-426614174000"
        ),
        QueryClassifications,
    ),
    tag = "engine",
)]
#[get("/classifications")]
pub async fn get_classifications(
    project_id: web::Path<String>,
    params: web::Query<QueryClassifications>,
    api_state: web::Data<ApiState>,
) -> PiResult<impl Responder> {
    let request_id = api_state.req_id.fetch_add(1);
    let project_id = project_id.into_inner();

    debug!(
        "API request {} for project {} to get classifications",
        request_id, project_id
    );

    // Subscribe to receive engine response
    let mut rx = api_state.api_channel_tx.subscribe();

    api_state.main_tx.send(PiEvent::APIRequest(
        project_id.clone(),
        EngineRequest {
            request_id: request_id.clone(),
            project_id: project_id.clone(),
            payload: EngineRequestPayload::GetClassifications,
        },
    ))?;

    debug!("Waiting for response for request {}", request_id);
    let mut response_opt: Option<EngineResponsePayload> = None;
    while let None = response_opt {
        match rx.recv().await {
            Ok(event) => match event {
                PiEvent::APIResponse(p_id, response) => {
                    if p_id == project_id && response.request_id == request_id {
                        response_opt = Some(response.payload.clone());
                    } else {
                    }
                }
                _ => {}
            },
            Err(_err) => {}
        }
    }

    debug!("Got response for request {}", request_id);
    match response_opt {
        Some(EngineResponsePayload::Classifications(classifications)) => {
            let filtered_classifications = if let Some(is_relevant) = params.is_relevant {
                classifications
                    .into_iter()
                    .filter(|classification| classification.is_relevant == is_relevant)
                    .collect::<Vec<_>>()
            } else {
                classifications
            };
            Ok(web::Json(EngineResponsePayload::Classifications(
                filtered_classifications,
            )))
        }
        _ => Ok(web::Json(EngineResponsePayload::Error(
            "Could not get a response".into(),
        ))),
    }
}

pub fn configure_api_engine(app_config: &mut utoipa_actix_web::service_config::ServiceConfig) {
    app_config.service(
        utoipa_actix_web::scope::scope("/engine/{project_id}")
            .service(get_labels)
            .service(get_nodes)
            .service(get_edges)
            .service(create_node)
            .service(create_edge)
            .service(search_results)
            .service(explore)
            .service(get_entities)
            .service(get_classifications),
    );
}

pub fn handle_engine_api_request(
    request: EngineRequest,
    engine: Arc<&Engine>,
    main_channel_tx: crossbeam_channel::Sender<PiEvent>,
) -> PiResult<()> {
    let response = match request.payload {
        EngineRequestPayload::Explore(optional_current_node_id) => {
            // The `Explore` request helps the UI show the graph in a way that makes it easy to visualize.
            // We start with nodes in a manner similar to how we process, and the UI can ask for further nodes.
            let starting_node = match optional_current_node_id {
                Some(current_node_id) => {
                    engine.get_node_by_id(&current_node_id).ok_or_else(|| {
                        PiError::InternalError("Cannot find given starting node".to_string())
                    })?
                }
                None => {
                    // When no starting node is given, we find the first objective node
                    let mut node_ids_with_label =
                        engine.get_node_ids_with_label(&NodeLabel::Objective);
                    node_ids_with_label.sort();

                    node_ids_with_label
                        .iter()
                        .find_map(|node_id| match engine.get_node_by_id(node_id) {
                            Some(arced_node) => Some(arced_node),
                            None => None,
                        })
                        .ok_or_else(|| {
                            PiError::InternalError(
                                "Could not find the starting Objective node".to_string(),
                            )
                        })?
                }
            };

            // With the starting node, we fetch nodes and edges.
            // We fetch 5 levels of nodes, and 4 levels of edges between them.
            let max_depth = 6;
            let node_labels_of_interest = [
                NodeLabel::Objective,
                NodeLabel::ProjectSettings,
                NodeLabel::CrawlerSettings,
                NodeLabel::ClassifierSettings,
                NodeLabel::Link,
                NodeLabel::Domain,
                NodeLabel::WebSearch,
            ];

            let mut node_ids_to_check: Vec<Vec<NodeId>> = vec![vec![starting_node.id]]; // By depth
            let mut node_ids: Vec<NodeId> = vec![starting_node.id];
            let mut nodes: Vec<ArcedNodeItem> = vec![starting_node];
            let mut edges: HashMap<NodeId, APINodeEdges> = HashMap::new();
            let mut sibling_nodes: Vec<Vec<NodeId>> = vec![];

            // Loop over nodes connected to the current node and add them to nodes and edges to explore.
            for depth in 0..max_depth {
                node_ids_to_check.push(vec![]);
                for node_id in node_ids_to_check[depth].clone().iter() {
                    let mut visited_sibling_nodes: HashMap<String, Vec<NodeId>> = HashMap::new();
                    match engine.get_connected_nodes(node_id) {
                        Ok(optional_edges) => {
                            match optional_edges {
                                Some(node_edges) => {
                                    // Read and populate the connected nodes
                                    for edge in node_edges.edges.iter() {
                                        if node_ids.contains(&edge.0) {
                                            continue;
                                        }

                                        match engine.get_node_by_id(&edge.0) {
                                            Some(node) => {
                                                // Are we interested in this node?
                                                match node_labels_of_interest
                                                    .iter()
                                                    .find(|label| node.labels.contains(label))
                                                {
                                                    Some(_) => {
                                                        node_ids.push(node.id);

                                                        node_ids_to_check[depth + 1].push(node.id);

                                                        // Save the node
                                                        nodes.push(node.clone());

                                                        // If this node is connected to the starting node using the same edge label
                                                        // as an earlier node, then it is a sibling of the starting node.
                                                        visited_sibling_nodes
                                                            .entry(
                                                                node.labels
                                                                    .iter()
                                                                    .sorted()
                                                                    .join(","),
                                                            )
                                                            .and_modify(|siblings| {
                                                                siblings.push(node.id);
                                                            })
                                                            .or_insert(vec![node.id]);
                                                    }
                                                    None => {}
                                                }
                                            }
                                            None => {}
                                        }
                                    }

                                    // For every group of siblings with length > 1, we insert into node_siblings
                                    for (_edge_label, siblings) in visited_sibling_nodes.iter() {
                                        if siblings.len() > 1 {
                                            sibling_nodes.push(siblings.clone());
                                        }
                                    }

                                    // Save all the edges
                                    edges.insert(
                                        node_id.clone(),
                                        APINodeEdges {
                                            edges: node_edges
                                                .edges
                                                .iter()
                                                .map(|x| (x.0, x.1.to_string()))
                                                .collect(),
                                            written_at: node_edges.written_at.timestamp_millis(),
                                        },
                                    );
                                }
                                None => {}
                            }
                        }
                        Err(_) => {}
                    }
                }
            }

            EngineResponsePayload::Explore(Explore {
                nodes: nodes.iter().map(|x| APINodeItem::from_node(x)).collect(),
                edges: APIEdges(edges),
                sibling_nodes,
            })
        }
        EngineRequestPayload::GetLabels => {
            let labels = engine.get_all_node_labels();
            EngineResponsePayload::Labels(labels.iter().map(|x| x.to_string()).collect())
        }
        EngineRequestPayload::GetNodesWithLabel(label) => {
            let mut node_ids_with_label =
                engine.get_node_ids_with_label(&NodeLabel::from_str(&label)?);
            node_ids_with_label.sort();
            let nodes: Vec<APINodeItem> = node_ids_with_label
                .iter()
                .filter_map(|node_id| match engine.get_node_by_id(node_id) {
                    Some(arced_node) => Some(APINodeItem::from_node(&arced_node)),
                    None => None,
                })
                .collect();
            EngineResponsePayload::Nodes(nodes)
        }
        EngineRequestPayload::GetNodesWithIds(mut node_ids) => {
            node_ids.sort();
            let mut nodes: Vec<APINodeItem> = vec![];
            for node_id in node_ids {
                if let Some(arced_node) = engine.get_node_by_id(&node_id) {
                    nodes.push(APINodeItem::from_node(&arced_node));
                }
            }
            EngineResponsePayload::Nodes(nodes)
        }
        EngineRequestPayload::GetAllNodes(since) => {
            let mut nodes: Vec<APINodeItem> = engine
                .get_all_nodes()
                .iter()
                .filter_map(|arced_node| {
                    // Check if node was written after the given `since` unix timestamp
                    // Compare at the millisecond level, since browser date objects
                    // do not support sub-millisecond precision
                    if arced_node.written_at.timestamp_millis() > since {
                        Some(APINodeItem::from_node(arced_node))
                    } else {
                        None
                    }
                })
                .collect();
            nodes.sort_by(|a, b| a.id.cmp(&b.id));

            EngineResponsePayload::Nodes(nodes)
        }
        EngineRequestPayload::GetAllEdges(since) => {
            let edges: HashMap<NodeId, APINodeEdges> = engine
                .get_all_edges()
                .iter()
                .filter_map(|(node_id, node_edges)| {
                    // Check if node_edges was written after the given `since` unix timestamp
                    // Compare at the millisecond level, since browser date objects
                    // do not support sub-millisecond precision
                    if node_edges.written_at.timestamp_millis() > since {
                        Some((
                            **node_id,
                            APINodeEdges {
                                edges: node_edges
                                    .edges
                                    .iter()
                                    .map(|x| (x.0, x.1.to_string()))
                                    .collect(),
                                written_at: node_edges.written_at.timestamp_millis(),
                            },
                        ))
                    } else {
                        None
                    }
                })
                .collect();

            EngineResponsePayload::Edges(APIEdges(edges))
        }
        EngineRequestPayload::CreateNode(node_write) => {
            let node_id = match node_write {
                NodeWrite::Link(link_write) => Link::add(
                    engine.clone(),
                    &link_write.url,
                    vec![NodeLabel::AddedByUser, NodeLabel::Link],
                    vec![],
                    true,
                )?,
                NodeWrite::SearchTerm(text) => engine
                    .get_or_add_node(
                        Payload::Text(text.to_string()),
                        vec![NodeLabel::AddedByUser, NodeLabel::SearchTerm],
                        true,
                        None,
                    )?
                    .get_node_id(),
                NodeWrite::Objective(text) => engine
                    .get_or_add_node(
                        Payload::Text(text.to_string()),
                        vec![NodeLabel::AddedByUser, NodeLabel::Objective],
                        true,
                        None,
                    )?
                    .get_node_id(),
                NodeWrite::ProjectSettings(project_settings_write) => engine
                    .get_or_add_node(
                        Payload::ProjectSettings(ProjectSettings {
                            only_extract_data_from_specified_links: project_settings_write
                                .extract_data_only_from_specified_links,
                            only_crawl_within_domains_of_specified_links: project_settings_write
                                .crawl_within_domains_of_specified_links,
                            only_crawl_direct_links_from_specified_links: project_settings_write
                                .crawl_direct_links_from_specified_links,
                        }),
                        vec![NodeLabel::AddedByUser, NodeLabel::ProjectSettings],
                        true,
                        None,
                    )?
                    .get_node_id(),
            };
            EngineResponsePayload::NodeCreatedSuccessfully(node_id)
        }
        EngineRequestPayload::CreateEdge(edge_write) => {
            engine.add_connection(edge_write.node_ids, edge_write.edge_labels)?;
            EngineResponsePayload::EdgeCreatedSuccessfully
        }
        EngineRequestPayload::Query(node_id) => match engine.get_node_by_id(&node_id) {
            Some(node) => {
                if node.labels.contains(&NodeLabel::SearchTerm) {
                    match &node.payload {
                        Payload::Text(_) => {
                            let mut results: Vec<NodeItem> =
                                SavedSearch::query(&node, engine.clone(), &node_id.into())?;
                            results.sort_by(|a, b| a.id.cmp(&b.id));

                            EngineResponsePayload::Nodes(
                                results
                                    .iter()
                                    .map(|node| APINodeItem::from_node(&Arc::new(node.clone())))
                                    .collect::<Vec<APINodeItem>>(),
                            )
                        }
                        _ => EngineResponsePayload::Error(format!(
                            "Query only works on search terms, not on {}",
                            node.payload.to_string()
                        )),
                    }
                } else {
                    EngineResponsePayload::Error(format!(
                        "Query only works on search terms, not on {}",
                        node.payload.to_string()
                    ))
                }
            }
            None => EngineResponsePayload::Error(format!("Node {} not found", node_id)),
        },
        EngineRequestPayload::GetEntities => {
            let mut grouped_entities = vec![];
            let mut web_page_node_ids = engine.get_node_ids_with_label(&NodeLabel::WebPage);
            web_page_node_ids.sort();
            for web_page_node_id in web_page_node_ids {
                let Some(web_page_node) = engine.get_node_by_id(&web_page_node_id) else {
                    continue;
                };
                let extracted_entities: Option<Vec<ExtractedEntity>> = engine
                    .get_connected_nodes(&web_page_node.id)?
                    .and_then(|edges| {
                        edges.edges.iter().find_map(|(id, label)| {
                            if *label != EdgeLabel::Suggests {
                                return None;
                            }
                            let node = engine.get_node_by_id(id)?;
                            if node.labels.contains(&NodeLabel::ExtractedNamedEntities) {
                                if let Payload::ExtractedNamedEntities(entities) = &node.payload {
                                    return Some(entities.clone());
                                }
                            }
                            None
                        })
                    });
                if let Some(mut extracted_entities) = extracted_entities {
                    grouped_entities.append(&mut extracted_entities);
                }
            }
            EngineResponsePayload::Entities(
                grouped_entities
                    .into_iter()
                    .fold(HashMap::new(), |mut acc, entity| {
                        let entry = acc
                            .entry(entity.entity_name.to_string())
                            .or_insert(HashSet::new());
                        entry.insert(entity.matching_text);
                        acc
                    })
                    .into_iter()
                    .map(|(entity_name, extracted_text)| EntityGroup {
                        entity_name,
                        extracted_text: extracted_text.into_iter().collect::<Vec<String>>(),
                    })
                    .collect(),
            )
        }
        EngineRequestPayload::GetClassifications => {
            let mut classified_items = vec![];
            let mut web_page_node_ids = engine.get_node_ids_with_label(&NodeLabel::WebPage);
            web_page_node_ids.sort();
            for web_page_node_id in web_page_node_ids {
                let Some(web_page_node) = engine.get_node_by_id(&web_page_node_id) else {
                    continue;
                };
                let Some(full_url) =
                    engine
                        .get_connected_nodes(&web_page_node.id)?
                        .and_then(|edges| {
                            edges.edges.iter().find_map(|(id, label)| {
                                if *label != EdgeLabel::ParentOf {
                                    return None;
                                }
                                let link_node = engine.get_node_by_id(id)?;
                                if !link_node.labels.contains(&NodeLabel::Link) {
                                    return None;
                                }
                                match &link_node.payload {
                                    Payload::Link(link) => {
                                        let domain_node = Domain::find_existing(
                                            engine.clone(),
                                            FindDomainOf::Node(*id),
                                        )
                                        .ok()
                                        .flatten()?;
                                        let domain_name =
                                            Domain::get_domain_name(&domain_node).ok()?;
                                        Some(format!(
                                            "https://{}{}",
                                            domain_name,
                                            link.get_full_link()
                                        ))
                                    }
                                    _ => None,
                                }
                            })
                        })
                else {
                    continue;
                };
                let Some(classification) =
                    engine
                        .get_connected_nodes(&web_page_node.id)?
                        .and_then(|edges| {
                            edges.edges.iter().find_map(|(id, label)| {
                                if *label != EdgeLabel::Classifies {
                                    return None;
                                }
                                let node = engine.get_node_by_id(id)?;
                                if node.labels.contains(&NodeLabel::Classification) {
                                    if let Payload::Classification(classification) = &node.payload {
                                        return Some(classification.clone());
                                    }
                                }
                                None
                            })
                        })
                else {
                    continue;
                };
                classified_items.push(ClassifiedItem {
                    url: full_url,
                    is_relevant: classification.is_relevant,
                    reason: classification.reason,
                    insight: classification.insight_if_classified_as_relevant,
                });
            }
            EngineResponsePayload::Classifications(classified_items)
        }
    };

    main_channel_tx.send(PiEvent::APIResponse(
        request.project_id,
        EngineResponse {
            request_id: request.request_id,
            payload: response,
        },
    ))?;

    Ok(())
}
