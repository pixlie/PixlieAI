use crate::{ExternalData, PiEvent};
use log::{debug, error};
use reqwest::StatusCode;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use url::Url;

struct FetchLog {
    last_fetched_at: Instant,
}

struct DomainLog {
    per_url: HashMap<String, FetchLog>,
    last_fetched_at: Instant,
}

type Logs = HashMap<String, DomainLog>;

enum CanCrawl {
    Yes,
    No(String),
}

fn can_crawl_domain(url: &str, logs: &mut Logs) -> CanCrawl {
    let parsed = match Url::parse(&url) {
        Ok(parsed) => parsed,
        Err(err) => {
            return CanCrawl::No(format!("Cannot parse URL {} to get domain: {}", &url, err))
        }
    };
    let domain = match parsed.domain() {
        Some(domain) => domain,
        None => return CanCrawl::No(format!("Cannot parse URL {} to get domain", &url)),
    };
    if domain.is_empty() {
        return CanCrawl::No(format!("Cannot parse URL {} to get domain", &url));
    }
    match logs.get(domain) {
        Some(domain_log) => {
            // Check the last fetch time for this domain. We do not want to fetch too often.
            if domain_log.last_fetched_at.elapsed().as_secs() > 2 {
                // We have fetched from this domain some time ago, we can fetch now
                CanCrawl::Yes
            } else {
                // We have fetched from this domain very recently, we can not fetch now
                debug!("Domain was recently fetched from, cannot fetch now");
                CanCrawl::No("Domain was recently fetched from, cannot fetch now".to_string())
            }
        }
        None => {
            // We have not fetched from this domain yet, we can fetch now
            logs.insert(
                domain.to_string(),
                DomainLog {
                    per_url: HashMap::new(),
                    last_fetched_at: Instant::now(),
                },
            );
            CanCrawl::Yes
        }
    }
}

enum FetchResult {
    Contents(StatusCode, String),
    Error(String),
}

async fn fetch_url(url: &str) -> FetchResult {
    match reqwest::Client::builder()
        .user_agent("Pixlie AI bot (https://pixlie.com)")
        .timeout(Duration::from_secs(3))
        .build()
    {
        Ok(client) => {
            let response = client.get(url).send().await;
            match response {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        match response.text().await {
                            Ok(contents) => FetchResult::Contents(status, contents),
                            Err(err) => FetchResult::Error(err.to_string()),
                        }
                    } else {
                        FetchResult::Error("Fetch response status is not success".to_string())
                    }
                }
                Err(err) => FetchResult::Error(format!("Error getting response: {}", err)),
            }
        }
        Err(err) => FetchResult::Error(format!("Error building reqwest client: {}", err)),
    }
}

pub fn fetcher_runtime(
    mut fetch_rx: tokio::sync::mpsc::Receiver<PiEvent>,
    main_tx: crossbeam_channel::Sender<PiEvent>,
) {
    // This function manages an asynchronous runtime and spawns a thread for each request
    let mut domain_logs: HashMap<String, DomainLog> = HashMap::new();
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
                    PiEvent::FetchRequest(project_id, id, url) => {
                        let fetch_response: PiEvent = {
                            match can_crawl_domain(&url, &mut domain_logs) {
                                CanCrawl::No(err) => PiEvent::FetchError(project_id, id, err),
                                CanCrawl::Yes => match fetch_url(&url).await {
                                    FetchResult::Contents(_status, contents) => {
                                        PiEvent::FetchResponse(
                                            project_id,
                                            id,
                                            url,
                                            ExternalData::Text(contents),
                                        )
                                    }
                                    FetchResult::Error(err) => {
                                        error!("Error fetching URL: {}", err);
                                        PiEvent::FetchError(project_id, id, err)
                                    }
                                },
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
