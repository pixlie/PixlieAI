use clap::Parser;
use config::{Cli, CrawlRequest};
use env_logger;
use log::error;
use log::info;
use rumqttc::v5::AsyncClient;
use rumqttc::v5::ConnectionError;
use rumqttc::v5::Event;
use rumqttc::v5::Incoming;
use rumqttc::v5::MqttOptions;
use serde_json;
use spider;
use spider::features::chrome_common::RequestInterceptConfiguration;
use spider::tokio;
use spider::website::Website;

pub mod config;

async fn crawl(website: Website) {
    match website
        .build()
    {
        Ok(mut website) => {
            let mut rx2 = website.subscribe(0).expect("sync feature required");

            tokio::spawn(async move {
                website.crawl().await;
            });

            while let Ok(res) = rx2.recv().await {
                match res.get_url_parsed() {
                    Some(parsed_url) => {
                        let url_path = parsed_url.path();
                        match res.get_bytes() {
                            Some(b) => {
                                info!("URL: {}\n\n{:?}", url_path, b);
                            }
                            _ => (),
                        }
                    }
                    _ => ()
                }
            }
        }
        _ =>  println!("Invalid website URL passed in. The url should start with http:// or https:// following the website domain ex: https://example.com.")
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let cli = Cli::parse();
    // let crawl_request = CrawlRequest
    let mqqt_opts = MqttOptions::new("pixlieai", cli.mqtt_host, cli.mqtt_port);
    let (_, mut eventloop) = AsyncClient::new(mqqt_opts, 10);

    if cli.verbose {
        use env_logger::Env;
        let env = Env::default()
            .filter_or("RUST_LOG", "info")
            .write_style_or("RUST_LOG_STYLE", "always");

        env_logger::init_from_env(env);
    }

    info!("Starting Crawler");
    loop {
        match eventloop.poll().await {
            Ok(event) => {
                if let Event::Incoming(Incoming::Publish(packet)) = event {
                    match serde_json::from_slice::<CrawlRequest>(packet.payload.as_ref()) {
                        Ok(crawl_request) => {
                            let mut website = Website::new(&crawl_request.url);

                            website
                                .with_respect_robots_txt(cli.respect_robots_txt)
                                .with_subdomains(crawl_request.subdomains)
                                .with_chrome_intercept(RequestInterceptConfiguration::new(true))
                                .with_danger_accept_invalid_certs(false)
                                .with_tld(false)
                                .with_user_agent(Some("PixlieAI Crawler"))
                                .with_delay(4)
                                .with_limit(100)
                                .with_depth(3);

                            crawl(website).await;
                        }
                        Err(e) => {
                            println!("Error parsing crawl request: {}", e);
                            continue;
                        }
                    };
                }
            }
            Err(e) => match e {
                ConnectionError::ConnectionRefused(_) => {
                    error!("Connection to MQTT server refused, is it running?");
                    break;
                }
                ConnectionError::Io(_) => {
                    error!("Connection to MQTT server failed, is it running?");
                    break;
                }
                _ => {
                    println!("Error polling eventloop: {}", e);
                    // Sleep for 1 second before trying again
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            },
        }
    }
}
