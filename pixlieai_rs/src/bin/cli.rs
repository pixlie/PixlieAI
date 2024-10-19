use pixlieai::{
    config::{get_cli_settings, Settings},
    engine::{Engine, Payload},
    entity::{fetchable::FetchStatus, web::Link},
};

#[tokio::main]
async fn main() {
    env_logger::init();
    // Log current directory
    let _settings: Settings = get_cli_settings().unwrap();

    let news = Link {
        url: "https://techcrunch.com/2024/10/07/ai-powered-critical-mineral-startup-kobold-metals-has-raised-491m-filings-reveal/".to_string(),
        fetched: FetchStatus::NotFetched
    };

    let engine = Engine::new();
    engine.add_node(Payload::Link(news)).await;
    engine.execute().await;
}
