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
pub struct Crawl {
    pub starting_urls: Vec<String>,
    pub web_search_keywords: Vec<String>,
    pub continue_crawl: Option<ContinueCrawl>,
}

// Features that are available in Pixlie for an AI agent
#[derive(Deserialize, TS)]
pub enum Feature {
    Crawl(Crawl),
    // ExtractNamedEntities(Vec<NamedEntity>),
}

#[derive(Deserialize, TS)]
pub struct LLMResponse {
    pub short_project_name: String,
    pub features: Vec<Feature>,
}
