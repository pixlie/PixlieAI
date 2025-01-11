use crossbeam_channel::unbounded;
use log::error;
use pixlieai::{
    api::api_manager, config::check_cli_settings, engine::manager::engine_manager, PiEvent,
};
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
    let (tx, rx) = unbounded::<PiEvent>();
    let (api_manager_tx, api_manager_rx) = (tx.clone(), rx.clone());
    thread_handles.push(thread::spawn(move || {
        match api_manager(api_manager_tx, api_manager_rx) {
            Ok(_) => {}
            Err(err) => {
                error!("Error with api manager: {}", err);
            }
        }
    }));

    let (engine_manager_tx, engine_manager_rx) = (tx.clone(), rx.clone());
    thread_handles.push(thread::spawn(move || {
        match engine_manager(engine_manager_tx, engine_manager_rx) {
            Ok(_) => {}
            Err(err) => {
                error!("Error with graph engine: {}", err);
            }
        }
    }));
    for thread_handle in thread_handles {
        thread_handle.join().unwrap();
    }
}
