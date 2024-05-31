use std::sync::Arc;

use crate::config::Config;

mod openai;

pub use openai::OpenAiBackend;

pub trait LlmBackend {
    type MR;

    async fn models(&self) -> Self::MR;

    fn from_config(config: &Config) -> Arc<Self>;
}
