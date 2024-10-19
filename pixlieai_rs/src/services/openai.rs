// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the Business Source License 1.1 (the "License");
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

/*!
Code incomplete
*/

impl LargeLanguageModel {
    pub fn get_models_for_openai() -> Vec<LargeLanguageModel> {
        let mut models: Vec<AIModel> = vec![];
        models.push(AIModel {
            label: "GPT-3.5 Turbo".to_string(),
            ai_provider: AIProvider::OpenAI,
            developer: Some(AIModelDeveloper::SameAsProvider),
            features: HashSet::from([AIModelFeatures::TextGeneration]),
            api_name: "gpt-3.5-turbo".to_string(),
            latest_version_api_name: Some("gpt-3.5-turbo-0125".to_string()),
            context_window: Some(16_385),
            ..Default::default()
        });
        models.push(AIModel {
            label: "GPT-4 Turbo".to_string(),
            ai_provider: AIProvider::OpenAI,
            developer: Some(AIModelDeveloper::SameAsProvider),
            features: HashSet::from([
                AIModelFeatures::TextGeneration,
                AIModelFeatures::ImageRecognition,
            ]),
            api_name: "gpt-4-turbo".to_string(),
            latest_version_api_name: Some("gpt-4-turbo-2024-04-09".to_string()),
            context_window: Some(128_000),
            ..Default::default()
        });
        models.push(AIModel {
            label: "GPT-4o".to_string(),
            ai_provider: AIProvider::OpenAI,
            developer: Some(AIModelDeveloper::SameAsProvider),
            features: HashSet::from([
                AIModelFeatures::TextGeneration,
                AIModelFeatures::ImageRecognition,
            ]),
            api_name: "gpt-4o".to_string(),
            latest_version_api_name: Some("gpt-4o-2024-05-13".to_string()),
            context_window: Some(128_000),
            ..Default::default()
        });
        models
    }
}
