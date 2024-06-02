use std::{io::Write, sync::Arc};

use crate::{apitype, config::Config, streamer::StreamWriter, streamer::StreamWriterBytes};

mod openai;

pub use openai::OpenAiBackend;

use openai_dive::v1::resources::chat::ChatMessage;

pub trait LlmBackend {
    type MR;

    async fn models(&self) -> Self::MR;

    fn from_config(config: &Config) -> Arc<Self>;

    async fn submit_prompt(
        &self,
        chat_messages: Vec<ChatMessage>,
        model: &str,
    ) -> apitype::ChatCompletionResponse;

    async fn submit_prompt_stream(
        &self,
        chat_messages: Vec<ChatMessage>,
        stream_writer: StreamWriterBytes,
        model: &str,
    );
}
