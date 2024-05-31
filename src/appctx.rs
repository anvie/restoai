use std::sync::Arc;

use crate::{config::Config, llm::LlmBackend, streamer::Streamer};

pub struct AppContext<T>
where
    T: LlmBackend,
{
    pub streamer: Arc<Streamer>,
    pub llm_backend: Arc<T>,
}

impl<T> AppContext<T>
where
    T: LlmBackend,
{
    pub fn new(streamer: Arc<Streamer>, llm_backend: Arc<T>) -> Arc<Self> {
        Arc::new(Self { streamer, llm_backend })
    }

    pub fn from_config(config: &Config) -> Arc<Self> {
        let llm_backend = T::from_config(&config);
        Self::new(Streamer::new(), llm_backend)
    }
}
