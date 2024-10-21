use super::NodeWorker;
use crate::{
    engine::{Engine, NodeId, Payload},
    entity::web::{CrawledWebPage, Link},
    error::{PiError, PiResult},
};
use chrono::Utc;
use log::error;
use reqwest::{blocking, IntoUrl};

// pub async fn helper_extract_tables(contents: String) -> PiResult<Vec<String>> {}

impl NodeWorker for Link {
    fn process(&mut self, engine: &Engine, node_id: &NodeId) {
        // Download the linked URL and add a new WebPage node
        if !self.is_fetched {
            match blocking::get(&self.url) {
                Ok(response) => match response.text() {
                    Ok(contents) => {
                        engine.add_related_node(
                            node_id,
                            Payload::FileHTML(CrawledWebPage {
                                contents,
                                is_scraped: false,
                            }),
                        );
                        self.is_fetched = true;
                    }
                    Err(err) => {
                        error!("Error fetching link: {}", err);
                    }
                },
                Err(err) => {
                    error!("Error fetching link: {}", err);
                }
            }
        }
    }
}

impl NodeWorker for CrawledWebPage {
    fn process(&mut self, engine: &Engine, node_id: &NodeId) {}
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use test_log::test;

//     #[test(tokio::test)]
//     async fn test_fetch_link() {
//         let link = "https://growthlist.co/funded-startups/".to_string();
//         let contents = helper_fetch_link(link).await.unwrap();
//         assert!(contents.contains("plantpathco.com"));
//         assert!(contents.contains("Agrim Wholesale"));
//     }

//     #[test(tokio::test)]
//     async fn test_extract_tables() {}
// }
