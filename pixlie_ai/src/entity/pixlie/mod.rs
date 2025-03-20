use serde::Deserialize;
use ts_rs::TS;

// #[derive(Deserialize, TS)]
// #[ts(export)]
// pub enum NamedEntity {
//     BlogPost,
//     JobPost,
//     People,
//     Company,
//     Event,
//     Date,
//     Location,
//     PressRelease,
//     Product,
//     Currency,
//     Price,
// }

#[derive(Deserialize, TS)]
pub enum ContinueCrawl {
    IfContentHasKeywords(Vec<String>),
    // IfContentClassifiedAs(Vec<String>),
    // IfNamedEntityExtracted(Vec<NamedEntity>),
}

#[derive(Deserialize, TS)]
pub struct CrawlSpecification {
    pub web_search_keywords_for_objective: Vec<String>,
    pub conditions_to_continue_crawling: Option<ContinueCrawl>,
}

// Features that are available in Pixlie for an AI agent
#[derive(Deserialize, TS)]
pub enum Feature {
    Crawler(CrawlSpecification),
    // NamedEntityExtraction(Vec<NamedEntity>),
}

#[derive(Deserialize, TS)]
pub struct LLMResponse {
    pub short_project_name_with_spaces: String,
    pub features_needed_to_accomplish_objective: Vec<Feature>,
}
