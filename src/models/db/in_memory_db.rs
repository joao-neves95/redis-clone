use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use tokio::sync::Mutex;

use super::{app_data::AppData, in_memory_record::InMemoryRecord};

pub(crate) const EMPTY_RDB_HEX_FILE: &[u8] = b"524544495330303131fa0972656469732d76657205372e322e30fa0a72656469732d62697473c040fa056374696d65c26d08bc65fa08757365642d6d656dc2b0c41000fa08616f662d62617365c000fff06e3bfec0ff5aa2";

#[derive(Debug)]
pub struct InMemoryDb {
    records: HashMap<String, InMemoryRecord>,
    app_data: AppData,
}

impl InMemoryDb {
    pub fn new(app_data: AppData) -> Result<Arc<Mutex<Self>>, Error> {
        Ok(Arc::new(Mutex::new(InMemoryDb {
            records: HashMap::<String, InMemoryRecord>::new(),
            app_data,
        })))
    }

    pub fn get_records_ref_mut(&mut self) -> &mut HashMap<String, InMemoryRecord> {
        &mut self.records
    }

    pub fn get_app_data_ref(&self) -> &AppData {
        &self.app_data
    }

    pub fn get_app_data_mut(&mut self) -> &mut AppData {
        &mut self.app_data
    }
}
