use crate::entity::web::link::Link;
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

// impl WorkflowProcess {
//     fn get_label(&self) -> String {
//         match self {
//             WorkflowProcess::Link => Link::get_label(),
//             WorkflowProcess::WebPage => WebPage::get_label(),
//         }
//     }

//     fn arguments_needed(&self) -> Vec<String> {
//         match self {
//             WorkflowProcess::Link => vec![Link::get_label()],
//             _ => vec![],
//         }
//     }

//     // fn check_arguments(&self) {}
// }

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
