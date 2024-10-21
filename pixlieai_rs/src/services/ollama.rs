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
    pub async fn get_models_for_ollama() -> Result<Vec<LargeLanguageModel>, DwataError> {
        // We get the list of models from the Ollama API running on localhost
        let mut models: Vec<AIModel> = vec![];
        let ollama_models: OllamaModelsAPIResponse =
            reqwest::get("http://localhost:11434/api/tags")
                .await?
                .json::<OllamaModelsAPIResponse>()
                .await?;
        for model in ollama_models.models {
            models.push(AIModel {
                label: model.name.clone(),
                ai_provider: AIProvider::Ollama,
                api_name: model.name.clone(),
                ..Default::default()
            });
        }
        Ok(models)
    }
}
