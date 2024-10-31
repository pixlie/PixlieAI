use pixlieai::{
    config::{Rule, RuleCondition},
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

    let link_extract_rule = Rule::new(
        "Link",
        "Extract a link to be crawled later if the following conditions are met",
        data_extraction_conditions.clone(),
    );
    let table_data_extract_rule = Rule::new(
        "Table",
        "Extract table data from the given table if the headings match the given conditions",
        data_extraction_conditions.clone(),
    );
    let entity_extract_rule = Rule::new(
        "Entity",
        "Extract entities from the given text if the following conditions are met",
        entity_extraction_conditions.clone(),
    );

    let mut engine = Engine::new();
    engine.add_node(Payload::Rule(link_extract_rule));
    engine.add_node(Payload::Rule(table_data_extract_rule));
    engine.add_node(Payload::Rule(entity_extract_rule));
    engine.add_node(Payload::Link(Link {
        url: "https://growthlist.co/funded-startups/".to_string(),
        // url: "http://localhost:4321/pixlieai-tests/webpage-with-table.html".to_string(),
        is_fetched: false,
    }));
    engine.execute();
}
