#[cfg(test)]
pub(crate) mod utils {
    use crate::{
        models::db::{AppData, InMemoryDb},
        DEFAULT_LISTENING_PORT,
    };

    use std::sync::Arc;

    use anyhow::Error;
    use tokio::sync::Mutex;

    pub(crate) fn create_test_mem_db() -> Result<Arc<Mutex<InMemoryDb>>, Error> {
        Ok(InMemoryDb::new(AppData::new_master(
            DEFAULT_LISTENING_PORT,
        )?)?)
    }
}
