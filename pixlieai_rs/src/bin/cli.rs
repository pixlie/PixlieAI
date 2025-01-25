use log::error;
use pixlieai::{
    api::api_manager, config::check_cli_settings, engine::manager::engine_manager, CommsChannel,
};
use std::env::var;
use std::thread;

fn main() {
    // Setup Sentry for error logging. The URL comes from environment variable
    match var("SENTRY_URL") {
        Ok(sentry_url) => {
            if sentry_url.contains("sentry.io") {
                let _sentry = sentry::init((
                    sentry_url,
                    sentry::ClientOptions {
                        release: sentry::release_name!(),
                        ..Default::default()
                    },
                ));
            }
        }
        Err(_) => {}
    }

    env_logger::init();
    match check_cli_settings() {
        Ok(_) => {}
        Err(err) => {
            error!("Error with settings check: {}", err);
            return;
        }
    }
    let mut thread_handles: Vec<thread::JoinHandle<()>> = Vec::new();
    let engine_ch = CommsChannel::new();
    let api_ch = CommsChannel::new();
    let engine_ch1 = engine_ch.clone();
    let api_ch1 = api_ch.clone();
    thread_handles.push(thread::spawn(move || {
        match api_manager(engine_ch1, api_ch1) {
            Ok(_) => {}
            Err(err) => {
                error!("Error with api manager: {}", err);
            }
        }
    }));

    thread_handles.push(thread::spawn(move || {
        match engine_manager(engine_ch, api_ch) {
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
