use super::web::{Link, WebPage};
use crate::engine::{Engine, NodeWorker, Payload};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Deserialize, Serialize, TS)]
pub enum WorkflowProcess {
    Link,
    WebPage,
}

pub enum WorkflowArguments {
    Link(Link),
}

impl WorkflowProcess {
    fn get_label(&self) -> String {
        match self {
            WorkflowProcess::Link => Link::get_label(),
            WorkflowProcess::WebPage => WebPage::get_label(),
        }
    }

    fn arguments_needed(&self) -> Vec<String> {
        match self {
            WorkflowProcess::Link => vec![Link::get_label()],
            _ => vec![],
        }
    }

    // fn check_arguments(&self) {}
}

// pub struct For {
//     each: WorkflowProcess,
// }

#[derive(Clone, Deserialize, Serialize, TS)]
pub enum WorkflowCondition {
    IfValueIsIn(Vec<String>),
}

#[derive(Clone, Deserialize, Serialize, TS)]
#[ts(export)]
pub struct WorkflowStep {
    pub process: WorkflowProcess,
    pub conditions: WorkflowCondition,
}

pub fn startup_funding_insights_app(engine: &mut Engine) {
    // let entity_extraction_conditions: Vec<RuleCondition> = [
    //     "Company",
    //     "Funding",
    //     "PreviousFunding",
    //     "TotalFunding",
    //     "Valuation",
    //     "FundingStage",
    //     "Investor",
    //     "Founder",
    // ]
    // .iter()
    // .map(|x| RuleCondition::IfContextIncludes(x.to_string()))
    // .collect();

    let link_extract_rule = WorkflowStep {
        process: WorkflowProcess::WebPage,
        conditions: WorkflowCondition::IfValueIsIn(vec![
            "Startup Funding".to_string(),
            "Startup Investment".to_string(),
            "Startup Product".to_string(),
        ]),
    };
    // let table_data_extract_rule = Rule::new(
    //     "Table",
    //     "Extract table data from the given table if the headings match the given conditions",
    //     data_extraction_conditions.clone(),
    // );
    // let entity_extract_rule = Rule::new(
    //     "Entity",
    //     "Extract entities from the given text if the following conditions are met",
    //     entity_extraction_conditions.clone(),
    // );
    engine.add_node(Payload::Step(link_extract_rule));
    // engine.add_node(Payload::Rule(table_data_extract_rule));
    // engine.add_node(Payload::Rule(entity_extract_rule));
    engine.add_node(Payload::Link(Link {
        url: "https://growthlist.co/funded-startups/".to_string(),
        ..Default::default()
    }));
}
