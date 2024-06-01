use std::sync::Arc;

use crate::{config::Config, llm::LlmBackend, streamer::Streamer};

pub struct AppContext<T>
where
    T: LlmBackend,
{
    pub streamer: Arc<Streamer>,
    pub llm_backend: Arc<T>,
    pub config: Config,
}

impl<T> AppContext<T>
where
    T: LlmBackend,
{
    pub fn new(streamer: Arc<Streamer>, llm_backend: Arc<T>, config: Config) -> Arc<Self> {
        Arc::new(Self {
            streamer,
            llm_backend,
            config,
        })
    }

    pub fn from_config(config: &Config) -> Arc<Self> {
        //let llm_backend = T::from_config(&config);
        Self::new(Streamer::new(), T::from_config(config), config.clone())
    }
}
