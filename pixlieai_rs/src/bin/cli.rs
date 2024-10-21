use pixlieai::{
    config::{get_cli_settings, Settings},
    engine::{Engine, Payload},
    entity::web::Link,
};

fn main() {
    env_logger::init();
    // Log current directory
    let _settings: Settings = get_cli_settings().unwrap();

    let news = Link {
        url: "https://growthlist.co/funded-startups/".to_string(),
        is_fetched: false,
    };

    let engine = Engine::new();
    engine.add_node(Payload::Link(news));
    engine.execute();
}
