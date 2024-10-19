use crate::error::{PiError, PiResult};
use reqwest::Client;

pub async fn helper_fetch_link(link: String) -> PiResult<String> {
    let client = Client::new();
    let response = client.get(link.clone()).send().await?;
    Ok(response.text().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_fetch_link() {
        let link = "https://pixlie.com".to_string();
        let contents = helper_fetch_link(link).await.unwrap();
        assert!(contents.contains("Pixlie"));
    }
}
