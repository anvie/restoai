use actix_web_lab::body::writer;
use futures::StreamExt;
use openai_dive::v1::{
    api::Client,
    resources::chat::{ChatCompletionChunkResponse, ChatCompletionParameters, ChatMessage, ChatMessageContent, Role},
};
use serde_json;
use std::{env, io::Write, sync::Arc};

use crate::config::Config;
use crate::llm::LlmBackend;
use crate::streamer::StreamWriter;
use crate::{
    apitype,
    endpoint::{self},
};

pub struct OpenAiBackend {
    //api_key: String,
    client: Arc<Client>,
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
            client: Arc::new(Client::new(api_key)),
        }
    }
}

impl LlmBackend for OpenAiBackend {
    type MR = apitype::ModelList;

    async fn models(&self) -> apitype::ModelList {
        trace!("Fetching models from OpenAI API");
        self.client.models().list().await.expect("Failed to get models").into()
    }

    fn from_config(config: &Config) -> Arc<Self> {
        Arc::new(OpenAiBackend::new(config.openai_api_key.clone()))
    }

    async fn submit_prompt(&self, chat_messages: Vec<ChatMessage>) -> apitype::ChatCompletionResponse {
        let mut messages = vec![ChatMessage {
            role: Role::System,
            content: ChatMessageContent::Text("You are a helpful assistant.".to_string()),
            ..Default::default()
        }];
        messages = [messages, chat_messages].concat();
        let parameters = ChatCompletionParameters {
            model: "gpt-3.5-turbo".to_string(),
            messages,
            ..Default::default()
        };
        debug!("Submitting prompt to OpenAI API:\n {:#?}", parameters);
        let response = self.client.chat().create(parameters).await.expect("Failed to get response");
        trace!("Response from OAI: {:#?}", response);
        response.into()
    }

    async fn submit_prompt_stream(&self, chat_messages: Vec<ChatMessage>, mut stream_writer: StreamWriter) {
        let mut messages = vec![ChatMessage {
            role: Role::System,
            content: ChatMessageContent::Text("You are a helpful assistant.".to_string()),
            ..Default::default()
        }];
        messages = [messages, chat_messages].concat();
        let parameters = ChatCompletionParameters {
            model: "gpt-3.5-turbo".to_string(),
            messages,
            ..Default::default()
        };

        debug!("Submitting prompt to OpenAI API:\n {:#?}", parameters);

        let client = self.client.clone();

        let mut resp_stream = client.chat().create_stream(parameters).await.expect("Failed to get response");

        while let Some(response) = resp_stream.next().await {
            let response: ChatCompletionChunkResponse = response.expect("Failed to get response");

            trace!("Response from OAI: {:#?}", response);

            let data = apitype::ChatCompletionChunkResponse {
                id: response.id.into(),
                choices: response.choices.clone().into_iter().map(|c| c.into()).collect(),
                created: response.created,
                object: response.object.into(),
                model: None,
                system_fingerprint: None,
            };

            stream_writer
                //.write(serde_json::to_string(&response).expect("Failed to serialize response"))
                .write(serde_json::to_string(&data).expect("Failed to serialize response"))
                .await
                .expect("Failed to write to stream");

            // for choice in response.choices.iter() {
            //     if let Some(content) = &choice.delta.content {
            //         stream_writer.write(&content).await.expect("Failed to write to stream");
            //     }
            // }
            //
            // response.choices.iter().for_each(async |choice| {
            //     if let Some(content) = &choice.delta.content {
            //         stream_writer.write(content).await
            //     }
            // });
        }
    }
}
