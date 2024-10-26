use pixlieai::{
    engine::{Engine, Payload},
    entity::web::Link,
};

fn main() {
    env_logger::init();
    // Log current directory

    let engine = Engine::new();
    engine.add_node(Payload::Link(Link {
        url: "https://growthlist.co/funded-startups/".to_string(),
        // url: "http://localhost:4321/pixlieai-tests/webpage-with-table.html".to_string(),
        is_fetched: false,
    }));
    engine.execute();
}
