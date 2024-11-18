use clap::Parser;
use serde::Deserialize;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Cli {
    /// Print page visited on standard output
    #[clap(short, long)]
    pub verbose: bool,

    /// Host on which MQTT server is running
    #[clap(long, default_value = "localhost")]
    pub mqtt_host: String,

    /// Port on which MQTT server is running
    #[clap(long, default_value_t = 1883)]
    pub mqtt_port: u16,

    /// Respect robots.txt file
    #[clap(short, long)]
    pub respect_robots_txt: bool,
}

#[derive(Deserialize)]
pub struct CrawlRequest {
    /// The website URL to crawl
    pub url: String,

    /// Allow sub-domain crawling
    pub subdomains: bool,
}
