use crate::error::{PiError, PiResult};
use log::error;
use std::collections::HashMap;
use std::sync::atomic::AtomicU32;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;

pub enum FetchEvent {
    // None,
    FetchRequest(u32, String),
    FetchResponse(u32, String, String),
    FetchError(u32, String),
}

// https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html#communicating-between-sync-and-async-code
pub struct FetchChannel {
    // The tx is used by the engine to pass the request to the fetch_manager
    // The rx side of this is sent only to the fetch_manager
    pub tx: tokio::sync::mpsc::Sender<FetchEvent>,
    // The engine listens to this rx
    pub rx: crossbeam_channel::Receiver<FetchEvent>,
}

pub struct Fetcher {
    fetch_id: AtomicU32,
    fetch_channel: Arc<RwLock<FetchChannel>>,
    fetch_callers_tx: Arc<RwLock<HashMap<u32, crossbeam_channel::Sender<FetchEvent>>>>,
}

impl Fetcher {
    pub fn new() -> Self {
        // The engine creates a fetch_manager for itself and returns the channel
        // There is only one fetch_manager active per engine at a time
        let (fetcher_request_tx, fetcher_request_rx) =
            tokio::sync::mpsc::channel::<FetchEvent>(100);
        let (fetcher_response_tx, fetcher_response_rx) =
            crossbeam_channel::unbounded::<FetchEvent>();
        thread::spawn(move || {
            fetcher_runtime(fetcher_request_rx, fetcher_response_tx).unwrap();
        });

        let fetch_channel = Arc::new(RwLock::new(FetchChannel {
            tx: fetcher_request_tx.clone(),
            rx: fetcher_response_rx.clone(),
        }));
        let fetch_callers_tx: Arc<RwLock<HashMap<u32, crossbeam_channel::Sender<FetchEvent>>>> =
            Arc::new(RwLock::new(HashMap::new()));
        {
            let fetch_channel = fetch_channel.clone();
            let fetch_callers_tx = fetch_callers_tx.clone();
            thread::spawn(move || {
                response_manager(fetch_channel, fetch_callers_tx).unwrap();
            });
        }

        Self {
            fetch_id: AtomicU32::new(1),
            fetch_channel,
            fetch_callers_tx,
        }
    }

    pub fn fetch(&self, url: String) -> PiResult<crossbeam_channel::Receiver<FetchEvent>> {
        let id = self
            .fetch_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let (fetch_caller_tx, fetch_caller_rx) = crossbeam_channel::unbounded::<FetchEvent>();
        match self.fetch_callers_tx.write() {
            Ok(mut fetch_callers_tx) => {
                fetch_callers_tx.insert(id, fetch_caller_tx.clone());
            }
            Err(err) => {
                error!("Error writing to fetch caller rx: {}", err);
            }
        }
        // drop(fetch_caller_tx);
        let tx = match self.fetch_channel.read() {
            Ok(ch) => ch.tx.clone(),
            Err(err) => {
                error!("Fetch channel is not available: {}", err);
                return Err(PiError::InternalError(
                    "Fetch channel is not available".to_string(),
                ));
            }
        };

        thread::spawn(
            move || match tx.blocking_send(FetchEvent::FetchRequest(id, url)) {
                Ok(_) => {}
                Err(err) => {
                    error!("Could not send fetch request across channel: {}", err);
                }
            },
        );
        Ok(fetch_caller_rx)
    }
}

fn fetcher_runtime(
    mut fetch_rx: tokio::sync::mpsc::Receiver<FetchEvent>,
    fetch_tx: crossbeam_channel::Sender<FetchEvent>,
) -> PiResult<()> {
    // This function manages an asynchronous runtime and spawns a thread for each request
    let rt = Runtime::new()?;

    rt.block_on(async {
        loop {
            let event = fetch_rx.recv().await;
            match event {
                Some(event) => match event {
                    // FetchEvent::None => {},
                    FetchEvent::FetchRequest(id, url) => {
                        let fetch_response = match reqwest::Client::builder()
                            .user_agent("Pixlie AI bot (https://pixlie.com)")
                            .timeout(Duration::from_secs(3))
                            .build()
                        {
                            Ok(client) => {
                                let response = client.get(&url).send().await;
                                match response {
                                    Ok(response) => {
                                        if response.status().is_success() {
                                            match response.text().await {
                                                Ok(contents) => {
                                                    FetchEvent::FetchResponse(id, url, contents)
                                                }
                                                Err(err) => {
                                                    error!("Error fetching URL: {}", err);
                                                    FetchEvent::FetchError(id, err.to_string())
                                                }
                                            }
                                        } else {
                                            error!("Error fetching URL: {}", response.status());
                                            FetchEvent::FetchError(
                                                id,
                                                response.status().to_string(),
                                            )
                                        }
                                    }
                                    Err(err) => {
                                        error!("Error fetching URL: {}", err);
                                        FetchEvent::FetchError(id, err.to_string())
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Error building reqwest client: {}", err);
                                FetchEvent::FetchError(id, err.to_string())
                            }
                        };

                        match fetch_tx.send(fetch_response) {
                            Ok(_) => {}
                            Err(err) => {
                                error!("Error sending PiEvent in Fetch channel: {}", err);
                            }
                        }
                    }
                    _ => {}
                },
                None => {}
            };
        }
    });
    Ok(())
}

fn response_manager(
    fetch_channel: Arc<RwLock<FetchChannel>>,
    fetch_callers_tx: Arc<RwLock<HashMap<u32, crossbeam_channel::Sender<FetchEvent>>>>,
) -> PiResult<()> {
    let rx = match fetch_channel.read() {
        Ok(ch) => ch.rx.clone(),
        Err(err) => {
            error!("Fetch channel is not available: {}", err);
            return Err(PiError::InternalError(
                "Fetch channel is not available".to_string(),
            ));
        }
    };
    // We reach the rx of the crossbeam channel and pass the response to the caller
    match rx.recv() {
        Ok(event) => match event {
            FetchEvent::FetchResponse(id, url, contents) => match fetch_callers_tx.write() {
                Ok(mut callers_rx) => match callers_rx.remove(&id) {
                    Some(tx) => match tx.send(FetchEvent::FetchResponse(id, url, contents)) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Error sending to caller rx: {}", err);
                        }
                    },
                    None => {
                        error!("Fetcher ID {} not found", id);
                    }
                },
                Err(err) => {
                    error!("Error reading fetch caller rx: {}", err);
                }
            },
            _ => {}
        },
        Err(err) => {
            error!("Error receiving from fetch channel: {}", err);
        }
    }
    Ok(())
}
