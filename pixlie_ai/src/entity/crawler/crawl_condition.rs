use crate::engine::node::NodeItem;
use crate::engine::Engine;
use crate::error::PiResult;
use crate::ExternalData;
use std::sync::Arc;

#[allow(dead_code)]
pub struct CrawlCondition;

#[allow(dead_code, unused_variables)]
impl CrawlCondition {
    pub fn process(
        &self,
        node: &NodeItem,
        engine: Arc<&Engine>,
        data_from_previous_request: Option<ExternalData>,
    ) -> PiResult<()> {
        Ok(())
    }
}
