// TODO: Remove the following when we start using this module
#![allow(dead_code)]

use crate::engine::node::NodeItem;
use crate::engine::Engine;
use crate::error::PiResult;
use crate::ExternalData;
use std::sync::Arc;

pub struct CrawlCondition;

impl CrawlCondition {
    pub fn process(
        &self,
        _node: &NodeItem,
        _engine: Arc<&Engine>,
        _data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        Ok(())
    }
}
