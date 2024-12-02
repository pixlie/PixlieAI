use log::error;
use pixlieai::{admin::admin_manager, config::check_cli_settings, engine::engine_manager};
use std::thread;

fn main() {
    env_logger::init();
    match check_cli_settings() {
        Ok(_) => {}
        Err(err) => {
            error!("Error with settings check: {}", err);
            return;
        }
    }
    let mut thread_handles: Vec<thread::JoinHandle<()>> = Vec::new();
    thread_handles.push(thread::spawn(|| match admin_manager() {
        Ok(_) => {}
        Err(err) => {
            error!("Error with admin manager: {}", err);
        }
    }));
    thread_handles.push(thread::spawn(|| match engine_manager() {
        Ok(_) => {}
        Err(err) => {
            error!("Error with graph engine: {}", err);
        }
    }));
    for thread_handle in thread_handles {
        thread_handle.join().unwrap();
    }
}
