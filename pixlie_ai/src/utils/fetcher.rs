use crate::{ExternalData, PiEvent};
use log::error;
use std::time::Duration;
use tokio::runtime::Runtime;

pub fn fetcher_runtime(
    mut fetch_rx: tokio::sync::mpsc::Receiver<PiEvent>,
    main_tx: crossbeam_channel::Sender<PiEvent>,
) {
    // This function manages an asynchronous runtime and spawns a thread for each request
    let rt = match Runtime::new() {
        Ok(rt) => rt,
        Err(err) => {
            error!("Error creating runtime: {}", err);
            return;
        }
    };

    rt.block_on(async {
        loop {
            let event = fetch_rx.recv().await;
            match event {
                Some(event) => match event {
                    // FetchEvent::None => {},
                    PiEvent::FetchRequest(project_id, id, url) => {
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
                                                Ok(contents) => PiEvent::FetchResponse(
                                                    project_id,
                                                    id,
                                                    url,
                                                    ExternalData::Text(contents),
                                                ),
                                                Err(err) => {
                                                    error!("Error fetching URL: {}", err);
                                                    PiEvent::FetchError(
                                                        project_id,
                                                        id,
                                                        err.to_string(),
                                                    )
                                                }
                                            }
                                        } else {
                                            error!("Error fetching URL: {}", response.status());
                                            PiEvent::FetchError(
                                                project_id,
                                                id,
                                                response.status().to_string(),
                                            )
                                        }
                                    }
                                    Err(err) => {
                                        error!("Error fetching URL: {}", err);
                                        PiEvent::FetchError(project_id, id, err.to_string())
                                    }
                                }
                            }
                            Err(err) => {
                                error!("Error building reqwest client: {}", err);
                                PiEvent::FetchError(project_id, id, err.to_string())
                            }
                        };

                        match main_tx.send(fetch_response) {
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
}
