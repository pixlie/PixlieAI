// Copyright 2024 Pixlie Web Solutions Pvt. Ltd.
// Licensed under the GNU General Public License version 3.0;
// You may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://www.pixlie.com/ai/license

/*!
Code incomplete
*/

impl LargeLanguageModel {
    pub fn get_models_for_groq() -> Vec<LargeLanguageModel> {
        let mut models: Vec<AIModel> = vec![];
        models.push(AIModel {
            label: "LLaMA3 8b".to_string(),
            ai_provider: AIProvider::Groq,
            developer: Some(AIModelDeveloper::Meta),
            features: HashSet::from([AIModelFeatures::TextGeneration]),
            api_name: "llama3-8b-8192".to_string(),
            context_window: Some(8_192),
            ..Default::default()
        });
        models.push(AIModel {
            label: "LLaMA3 70b".to_string(),
            ai_provider: AIProvider::Groq,
            developer: Some(AIModelDeveloper::Meta),
            features: HashSet::from([AIModelFeatures::TextGeneration]),
            api_name: "llama3-70b-8192".to_string(),
            context_window: Some(8_192),
            ..Default::default()
        });
        models.push(AIModel {
            label: "Mixtral 8x7b".to_string(),
            ai_provider: AIProvider::Groq,
            developer: Some(AIModelDeveloper::Mistral),
            features: HashSet::from([AIModelFeatures::TextGeneration]),
            api_name: "mixtral-8x7b-32768".to_string(),
            context_window: Some(32_768),
            ..Default::default()
        });
        models.push(AIModel {
            label: "Gemma 7b".to_string(),
            ai_provider: AIProvider::Groq,
            developer: Some(AIModelDeveloper::Google),
            features: HashSet::from([AIModelFeatures::TextGeneration]),
            api_name: "gemma-7b-it".to_string(),
            context_window: Some(8_192),
            ..Default::default()
        });
        models
    }
}
