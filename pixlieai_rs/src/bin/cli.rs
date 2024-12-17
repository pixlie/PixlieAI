use log::error;
use pixlieai::{
    admin::admin_manager, api::api_manager, config::check_cli_settings, engine::engine_manager,
    PiCliEvent,
};
use std::{sync::mpsc, thread};

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
    let (tx, rx) = mpsc::channel::<PiCliEvent>();
    thread_handles.push(thread::spawn(|| match admin_manager() {
        Ok(_) => {}
        Err(err) => {
            error!("Error with admin manager: {}", err);
        }
    }));

    let api_manager_tx = tx.clone();
    thread_handles.push(thread::spawn(move || match api_manager(api_manager_tx) {
        Ok(_) => {}
        Err(err) => {
            error!("Errow with api manager: {}", err);
        }
    }));

    // thread_handles.push(thread::spawn(move || match engine_manager(rx) {
    //     Ok(_) => {}
    //     Err(err) => {
    //         error!("Error with graph engine: {}", err);
    //     }
    // }));
    for thread_handle in thread_handles {
        thread_handle.join().unwrap();
    }
}
