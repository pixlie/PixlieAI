use pixlieai::{
    config::{Rule, RuleCondition, Rules},
    engine::{Engine, Payload},
    entity::web::Link,
};

fn main() {
    env_logger::init();

    let data_extraction_conditions: Vec<RuleCondition> =
        ["Startup Funding", "Startup Investment", "Startup Product"]
            .iter()
            .map(|x| RuleCondition::IfContextIncludes(x.to_string()))
            .collect();
    let entity_extraction_conditions: Vec<RuleCondition> = [
        "Company",
        "Funding",
        "PreviousFunding",
        "TotalFunding",
        "Valuation",
        "FundingStage",
        "Investor",
        "Founder",
    ]
    .iter()
    .map(|x| RuleCondition::IfContextIncludes(x.to_string()))
    .collect();

    let link_extract_rule = Rule::new("Link", "Extract a link", data_extraction_conditions.clone());
    let table_data_extract_rule = Rule::new(
        "Table",
        "Extract table data",
        data_extraction_conditions.clone(),
    );
    let entity_extract_rule = Rule::new(
        "Entity",
        "Extract entities",
        entity_extraction_conditions.clone(),
    );
    let rules = Rules {
        description: "Create a knowledge graph about startup funding by crawling websites related to startups, investors, startup news, etc.".to_string(),
        rules: vec![
            link_extract_rule,
            table_data_extract_rule,
            entity_extract_rule,
        ],
    };

    let mut engine = Engine::new();
    engine.add_node(Payload::Link(Link {
        url: "https://growthlist.co/funded-startups/".to_string(),
        // url: "http://localhost:4321/pixlieai-tests/webpage-with-table.html".to_string(),
        is_fetched: false,
    }));
    engine.execute();
}
