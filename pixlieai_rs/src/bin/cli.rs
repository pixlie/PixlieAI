use log::{error, info};
use pixlieai::{
    config::{get_cli_settings, Settings},
    entity::startup::FundingNews,
    provider::{anthropic, gliner, EntityExtractionProvider},
};

static SAMPLE_NEWS_TITLE: &str = r#"
KoBold Metals, which uses AI to help find critical minerals for the energy transition, raises $491M
"#;

static SAMPLE_NEWS_BODY: &str = r#"
Earlier this year, KoBold Metals found what might be one of the largest high-grade copper deposits of all time, with the potential to produce hundreds of thousands of metric tons per year, the company’s CEO said.
Now, just eight months later, KoBold is close to raising over half a billion dollars. The funding should help the company develop the massive copper resource while moving forward on its other exploration projects, which number in the dozens.
The mineral discovery startup has already raised $491 million of a targeted $527 million round, according to an SEC filing. Its previous round of $195 million valued the company at $1 billion post-money, according to PitchBook. The startup is reportedly aiming for a $2 billion valuation for the current round.
The company did not immediately reply to questions.
KoBold uses artificial intelligence to sift through enormous amounts of data in a quest to find mineral deposits that can help drive the energy transition. In addition to copper, the company searches for lithium, nickel, and cobalt.
Initially, the company was focused solely on discovery. Prospecting for minerals has historically been an endeavor fraught with risk. The rule of thumb is that for every 1,000 attempts to find a deposit, only about three tend to be successful. KoBold was betting that AI would be able to parse data and find trends that would lead to greater success rates.
With the enormous copper deposit in Zambia, Kobold appears to have delivered on its early promise. The company has about 60 other exploration projects underway, and in a strategic shift, KoBold has said it intends to develop the Zambia resource itself, an undertaking that reportedly will cost around $2.3 billion.
KoBold’s previous investors include Bill Gates, Jeff Bezos, Jack Ma, Andreessen Horowitz, and Breakthrough Energy Ventures.
"#;

#[tokio::main]
async fn main() {
    env_logger::init();
    // Log current directory
    let settings: Settings = get_cli_settings().unwrap();

    let startup_news = FundingNews {
        url: "https://techcrunch.com/2024/10/07/ai-powered-critical-mineral-startup-kobold-metals-has-raised-491m-filings-reveal/".to_string(),
        title: SAMPLE_NEWS_TITLE.to_string(),
        body_text: SAMPLE_NEWS_BODY.to_string(),
    };
    let provider: EntityExtractionProvider = EntityExtractionProvider::Gliner;

    let entities = match provider {
        EntityExtractionProvider::Gliner => {
            // Use GLiNER
            gliner::extract_entities(&startup_news, &settings.path_to_gliner_home).await
        }
        EntityExtractionProvider::Anthropic => {
            // Use Anthropic
            anthropic::extract_entities(&startup_news, &settings.anthropic_api_key).await
        }
    };
    // Log the entities
    match entities {
        Ok(entities) => {
            info!(
                "Extracted entities:\n{}",
                entities
                    .iter()
                    .map(|x| format!("{},{}", x.label.to_string(), x.matching_text.as_str()))
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        }
        Err(err) => {
            error!("Error extracting entities: {}", err);
        }
    }
}
