use crate::{CrawlOrAPIRequest, FetchError, FetchResponse, InternalFetchRequest, PiEvent};
use log::{debug, error, info};
use reqwest::header::HeaderMap;
use reqwest::{Client, Method, Request, RequestBuilder, StatusCode, Url};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;

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

fn check_logs(domain: &str, url: &str, logs: &mut Logs) -> CanCrawl {
    match logs.get_mut(domain) {
        Some(domain_log) => {
            // Check the last fetch time for this domain. We do not want to fetch too often.
            if domain_log.last_fetched_at.elapsed().as_secs() > 2 {
                // We have fetched from this domain some time ago, let's check the URL logs
                match domain_log.per_url.get(url) {
                    Some(url_log) => {
                        // Check the last fetch time for this URL. We do not want to fetch too often.
                        if url_log.last_fetched_at.elapsed().as_secs() > 2 {
                            // We have fetched from this URL some time ago, we can fetch now
                            debug!("URL {} was recently fetched from, cannot fetch now", url);
                            CanCrawl::No(format!(
                                "URL {} was recently fetched from, cannot fetch now",
                                url
                            ))
                        } else {
                            // We have fetched from this URL recently, let's update the last fetched time
                            domain_log.per_url.insert(
                                url.to_string(),
                                FetchLog {
                                    last_fetched_at: Instant::now(),
                                },
                            );
                            CanCrawl::Yes
                        }
                    }
                    None => {
                        // We have not fetched from this URL yet, we can fetch now
                        domain_log.per_url.insert(
                            url.to_string(),
                            FetchLog {
                                last_fetched_at: Instant::now(),
                            },
                        );
                        CanCrawl::Yes
                    }
                }
            } else {
                // We have fetched from this domain very recently, we can not fetch now
                debug!(
                    "Domain {} was recently fetched from, cannot fetch now",
                    domain
                );
                CanCrawl::No(format!(
                    "Domain {} was recently fetched from, cannot fetch now",
                    domain
                ))
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

async fn fetch(request: InternalFetchRequest) -> FetchResult {
    let url = request.crawl_or_api_request.get_url();
    let client = match Client::builder()
        .user_agent("Pixlie AI bot (https://pixlie.com)")
        .timeout(Duration::from_secs(match request.crawl_or_api_request {
            CrawlOrAPIRequest::Crawl(_) => 5,
            CrawlOrAPIRequest::API(_) => 60,
        }))
        .build()
    {
        Ok(client) => client,
        Err(err) => {
            return FetchResult::Error(format!(
                "Error building client to fetch URL {}: {}",
                &url, err
            ));
        }
    };
    let url = match Url::parse(&url) {
        Ok(url) => url,
        Err(err) => {
            return FetchResult::Error(format!("Error parsing URL {} to fetch URL: {}", &url, err));
        }
    };

    let request_builder = RequestBuilder::from_parts(client, Request::new(request.method, url));
    let request_builder = request_builder.headers(request.headers);
    let request_builder = match request.body {
        Some(body) => request_builder.body(body),
        None => request_builder,
    };
    match request_builder.send().await {
        Ok(response) => {
            let status = response.status();
            if status.is_success() {
                match response.text().await {
                    Ok(contents) => FetchResult::Contents(status, contents),
                    Err(err) => FetchResult::Error(err.to_string()),
                }
            } else {
                FetchResult::Error(format!(
                    "Fetch response status is not success, got response {}",
                    response.text().await.unwrap_or_else(|_| "".to_string())
                ))
            }
        }
        Err(error) => {
            info!("Error {:?}", error);
            FetchResult::Error(format!("Error getting response: {}", error))
        }
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
        async fn make_request(request: InternalFetchRequest) -> PiEvent {
            match fetch(request.clone()).await {
                FetchResult::Contents(_status, contents) => PiEvent::FetchResponse(FetchResponse {
                    project_id: request.project_id.clone(),
                    node_id: request.node_id,
                    url: request.crawl_or_api_request.get_url(),
                    contents,
                }),
                FetchResult::Error(err) => PiEvent::FetchError(FetchError {
                    project_id: request.project_id.clone(),
                    node_id: request.node_id,
                    error: err,
                }),
            }
        }

        loop {
            let event = fetch_rx.recv().await;
            match event {
                Some(event) => match event {
                    PiEvent::FetchRequest(request) => {
                        let fetch_response: PiEvent = match &request.crawl_or_api_request {
                            CrawlOrAPIRequest::Crawl(crawl_request) => match check_logs(
                                &crawl_request.domain,
                                &crawl_request.url,
                                &mut domain_logs,
                            ) {
                                CanCrawl::No(err) => PiEvent::FetchError(FetchError {
                                    project_id: request.project_id.clone(),
                                    node_id: request.node_id,
                                    error: err,
                                }),
                                CanCrawl::Yes => make_request(request).await,
                            },
                            CrawlOrAPIRequest::API(_) => make_request(request).await,
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
