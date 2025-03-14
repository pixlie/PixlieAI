use crate::error::{PiError, PiResult};
use crate::FetchRequest;

pub trait LLM {
    fn get_prompt_for_objective(_pixlie_schema: &String, _objective: &String) -> PiResult<String> {
        Err(PiError::NotAvailable(
            "LLM does not work with objective".to_string(),
        ))
    }

    fn get_request(
        _pixlie_schema: &String,
        _objective: &String,
        _calling_node_id: u32,
    ) -> PiResult<FetchRequest> {
        Err(PiError::NotAvailable(
            "LLM does not work with objective".to_string(),
        ))
    }
}
