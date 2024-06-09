use actix_web_lab::body::writer;
use futures::StreamExt;
use openai_dive::v1::{
    api::Client,
    resources::chat::{
        ChatCompletionChunkResponse, ChatCompletionParameters, ChatMessage, ChatMessageContent,
        Role,
    },
};
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
    pub fn new<TStr: ToString>(api_key: Option<TStr>, base_url: &str) -> Self {
        let api_key: String = api_key
            //.map(|k| k.to_string())
            .map_or_else(
                || env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"),
                |d| d.to_string(),
            );

        debug!("Creating OpenAI backend with base URL: {}", base_url);
        debug!("  with API key: {}", api_key);

        OpenAiBackend {
            //api_key,
            //client: Arc::new(Client::new(api_key)),
            client: Arc::new(Client {
                http_client: reqwest::Client::new(),
                base_url: base_url.to_string(),
                api_key,
                organization: None,
                project: None,
            }),
        }
    }

    fn system_prompt_from_model(&self, model: &str) -> ChatMessageContent {
        if model == "programmer" {
            ChatMessageContent::Text("You are top notch software engineer in the world, you can give recommendation and best practice in programming and will give concise \
                and optimized code example when needed. And always response in Bahasa Indonesia.".to_string())
        } else if model == "sysadmin" {
            ChatMessageContent::Text("You are top notch sysadmin in the world, you can give recommendation and best practice in system administration and devops, and will give \
                concise and optimized code example when needed. And always response in Bahasa Indonesia.".to_string())
        } else {
            ChatMessageContent::Text("You are a helpful assistant.".to_string())
        }
    }

    fn build_prompt(
        &self,
        chat_messages: Vec<ChatMessage>,
        model: &str,
    ) -> ChatCompletionParameters {
        let mut messages = vec![ChatMessage {
            role: Role::System,
            //content: ChatMessageContent::Text("You are a helpful assistant.".to_string()),
            content: self.system_prompt_from_model(model),
            ..Default::default()
        }];
        // remove system messages from user
        let chat_messages = chat_messages
            .into_iter()
            .filter(|m| m.role != Role::System)
            .collect();
        messages = [messages, chat_messages].concat();
        ChatCompletionParameters {
            model: env::var("OAI_MODEL_NAME").expect("OAI_MODEL_NAME not set"),
            messages,
            ..Default::default()
        }
    }
}

impl LlmBackend for OpenAiBackend {
    type MR = apitype::ModelList;

    async fn models(&self) -> apitype::ModelList {
        trace!("Fetching models from OpenAI API");
        self.client
            .models()
            .list()
            .await
            .expect("Failed to get models")
            .into()
    }

    fn from_config(config: &Config) -> Arc<Self> {
        env::set_var("OAI_MODEL_NAME", &config.llm_model_name);
        Arc::new(OpenAiBackend::new(
            config.openai_api_key.clone(),
            &config.llm_api_url,
        ))
    }

    async fn submit_prompt(
        &self,
        chat_messages: Vec<ChatMessage>,
        model: &str,
    ) -> apitype::ChatCompletionResponse {
        // let mut messages = vec![ChatMessage {
        //     role: Role::System,
        //     content: ChatMessageContent::Text("You are a helpful assistant.".to_string()),
        //     ..Default::default()
        // }];
        // messages = [messages, chat_messages].concat();
        // let parameters = ChatCompletionParameters {
        //     model: "gpt-3.5-turbo".to_string(),
        //     messages,
        //     ..Default::default()
        // };
        let parameters = self.build_prompt(chat_messages, model);
        //debug!("Submitting prompt to OpenAI API:\n {:#?}", parameters);
        let response = self
            .client
            .chat()
            .create(parameters)
            .await
            .expect("Failed to get response");
        debug!("Response from backend: {:#?}", response);
        response.into()
    }

    async fn submit_prompt_stream(
        &self,
        chat_messages: Vec<ChatMessage>,
        mut stream_writer: StreamWriter,
        model: &str,
    ) {
        // let mut messages = vec![ChatMessage {
        //     role: Role::System,
        //     content: ChatMessageContent::Text("You are a helpful assistant.".to_string()),
        //     ..Default::default()
        // }];
        // messages = [messages, chat_messages].concat();
        // let parameters = ChatCompletionParameters {
        //     model: "gpt-3.5-turbo".to_string(),
        //     messages,
        //     ..Default::default()
        // };
        //

        let parameters = self.build_prompt(chat_messages, model);

        debug!(
            "parameters:\n {}",
            serde_json::to_string_pretty(&parameters).unwrap_or("error to_string".to_string())
        );

        debug!(
            "Submitting prompt to OpenAI compatible server: {}",
            self.client.base_url
        );

        let client = self.client.clone();

        let mut resp_stream = client
            .chat()
            .create_stream(parameters)
            .await
            .expect("Failed to get response");

        while let Some(response) = resp_stream.next().await {
            let response: ChatCompletionChunkResponse = response.expect("Failed to get response");

            debug!("Response from backend: {:#?}", response);

            let data = apitype::ChatCompletionChunkResponse {
                id: response.id,
                choices: response
                    .choices
                    .clone()
                    .into_iter()
                    .map(|c| c.into())
                    .collect(),
                created: response.created,
                object: response.object,
                model: Some(model.to_string()),
                system_fingerprint: None,
            };

            stream_writer
                //.write(serde_json::to_string(&response).expect("Failed to serialize response"))
                .write(serde_json::to_string(&data).expect("Failed to serialize response"))
                .await
                .expect("Failed to write to stream");
        }
    }
}
