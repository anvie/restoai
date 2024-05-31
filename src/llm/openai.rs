use actix_web_lab::body::writer;
use openai_dive::v1::api::Client;
use serde_json;
use std::{env, sync::Arc};

use crate::config::Config;
use crate::endpoint::ModelList;
use crate::llm::LlmBackend;

pub struct OpenAiBackend {
    //api_key: String,
    client: Client,
}

impl OpenAiBackend {
    pub fn new<TStr: ToString>(api_key: Option<TStr>) -> Self {
        let api_key: String = api_key
            //.map(|k| k.to_string())
            .map_or_else(
                || env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
                |d| d.to_string(),
            );
        OpenAiBackend {
            //api_key,
            client: Client::new(api_key),
        }
    }
}

impl LlmBackend for OpenAiBackend {
    type MR = ModelList;

    async fn models(&self) -> ModelList {
        //vec!["gpt-3.5-turbo".to_string()]

        // Get the list of models from the OpenAI API.
        self.client.models().list().await.expect("Failed to get models")
        // .list()
        // .await
        // .expect("Failed to get models")
        // .data
        // .into_iter()
        // .map(|m| serde_json::from_str(m).unwrap())
    }

    fn from_config(config: &Config) -> Arc<Self> {
        Arc::new(OpenAiBackend::new(config.openai_api_key.clone()))
    }
}
