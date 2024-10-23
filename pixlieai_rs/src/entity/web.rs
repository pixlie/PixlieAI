use crate::{
    engine::{Engine, NodeId, Payload},
    services::EntityExtraction,
    workers::NodeWorker,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Link {
    pub url: String,
    pub is_fetched: bool,
}

impl Link {
    pub fn new(url: String) -> Link {
        Link {
            url,
            is_fetched: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct WebPage {
    pub contents: String,
    pub is_scraped: bool,
}

static WEBPAGE_EXTRACT_LABELS: &str = r#"
[
    Company,
    Funding,
    PreviousFunding,
    TotalFunding,
    Valuation,
    FundingStage,
    Investor,
    Founder,
]
"#;

impl EntityExtraction for WebPage {
    fn get_labels_to_extract(&self) -> Vec<String> {
        serde_yaml::from_str(WEBPAGE_EXTRACT_LABELS).unwrap()
    }

    fn get_payload(&self) -> String {
        self.contents.clone()
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{
//         config::get_test_settings,
//         entity::content::Paragraph,
//         services::{
//             anthropic::{self, extract_entities},
//             gliner, EntityExtractionProvider,
//         },
//     };
//     use log::{error, info};
//     use test_log::test;

//     static SAMPLE_NEWS_TITLE: &str = r#"
// KoBold Metals, which uses AI to help find critical minerals for the energy transition, raises $491M
// "#;

//     static SAMPLE_NEWS_BODY: &str = r#"
// Earlier this year, KoBold Metals found what might be one of the largest high-grade copper deposits of all time, with the potential to produce hundreds of thousands of metric tons per year, the company’s CEO said.
// Now, just eight months later, KoBold is close to raising over half a billion dollars. The funding should help the company develop the massive copper resource while moving forward on its other exploration projects, which number in the dozens.
// The mineral discovery startup has already raised $491 million of a targeted $527 million round, according to an SEC filing. Its previous round of $195 million valued the company at $1 billion post-money, according to PitchBook. The startup is reportedly aiming for a $2 billion valuation for the current round.
// The company did not immediately reply to questions.
// KoBold uses artificial intelligence to sift through enormous amounts of data in a quest to find mineral deposits that can help drive the energy transition. In addition to copper, the company searches for lithium, nickel, and cobalt.
// Initially, the company was focused solely on discovery. Prospecting for minerals has historically been an endeavor fraught with risk. The rule of thumb is that for every 1,000 attempts to find a deposit, only about three tend to be successful. KoBold was betting that AI would be able to parse data and find trends that would lead to greater success rates.
// With the enormous copper deposit in Zambia, Kobold appears to have delivered on its early promise. The company has about 60 other exploration projects underway, and in a strategic shift, KoBold has said it intends to develop the Zambia resource itself, an undertaking that reportedly will cost around $2.3 billion.
// KoBold’s previous investors include Bill Gates, Jeff Bezos, Jack Ma, Andreessen Horowitz, and Breakthrough Energy Ventures.
// "#;

//     #[test(tokio::test)]
//     async fn test_extract_entities_from_startup_news() {
//         let startup_news = CrawledWebPage {
//             meta_keywords: vec![],
//             meta_description: None,
//             title: SAMPLE_NEWS_TITLE.to_string(),
//             body: SAMPLE_NEWS_BODY.to_string(),
//         };
//         let settings = get_test_settings().unwrap();
//         let provider: EntityExtractionProvider = EntityExtractionProvider::Gliner;

//         let entities = match provider {
//             EntityExtractionProvider::Gliner => {
//                 // Use GLiNER
//                 gliner::extract_entities(&startup_news, &settings.path_to_gliner_home).await
//             }
//             EntityExtractionProvider::Anthropic => {
//                 // Use Anthropic
//                 anthropic::extract_entities(&startup_news, &settings.anthropic_api_key).await
//             }
//         };

//         match entities {
//             Ok(entities) => {
//                 // Log the entities
//                 info!(
//                     "Extracted entities:\n{}",
//                     entities
//                         .iter()
//                         .map(|x| format!("{},{}", x.label, x.matching_text))
//                         .collect::<Vec<String>>()
//                         .join("\n")
//                 );
//                 assert!(entities.len() > 8);
//                 assert!(entities
//                     .iter()
//                     .any(|x| x.label == "Funding" && x.matching_text.contains("491M")));
//             }
//             Err(err) => {
//                 error!("Error extracting entities: {}", err);
//             }
//         };
//     }
// }
