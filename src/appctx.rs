use std::sync::{Arc, Mutex};

use pickledb::PickleDb;

use crate::{config::Config, llm::LlmBackend};

pub struct AppContext<T>
where
    T: LlmBackend,
{
    pub llm_backend: Arc<T>,
    pub config: Config,
    pub db: Arc<Mutex<PickleDb>>,
}

impl<T> AppContext<T>
where
    T: LlmBackend,
{
    pub fn new(llm_backend: Arc<T>, config: Config) -> Arc<Self> {
        let path = "restoai.db";

        // check if db exists
        let db = if !std::path::Path::new(path).exists() {
            PickleDb::new_json(path, pickledb::PickleDbDumpPolicy::AutoDump)
        } else {
            PickleDb::load_json(path, pickledb::PickleDbDumpPolicy::AutoDump)
                .expect("Failed to load db")
        };

        let db = Arc::new(Mutex::new(db));
        Arc::new(Self {
            llm_backend,
            config,
            db,
        })
    }

    pub fn from_config(config: &Config) -> Arc<Self> {
        Self::new(T::from_config(config), config.clone())
    }
}
