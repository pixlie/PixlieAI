use crate::error::PiResult;
use crate::PiChannel;
use tokio::runtime::Runtime;

pub fn fetch_manager(_pi_channel: PiChannel) -> PiResult<()> {
    // This function manages an asynchronous runtime and spawns a thread for each request
    let _rt = Runtime::new()?;
    Ok(())
}
