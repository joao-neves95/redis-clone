use std::{collections::HashMap, sync::Arc, time::SystemTime};

use anyhow::Error;
use tokio::sync::Mutex;

use crate::utils::pseudo_random_ascii_alphanumeric;

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

    pub fn get_app_data_ref_mut(&mut self) -> &mut AppData {
        &mut self.app_data
    }
}

#[derive(Debug)]
pub struct InMemoryRecord {
    pub value: String,
    pub last_update_time: SystemTime,
    pub expire_milli: Option<u128>,
}

impl InMemoryRecord {
    pub fn new(value: String, expire_milli: Option<u128>) -> Self {
        InMemoryRecord {
            value,
            // No need for UTC. This is just an internal date.
            last_update_time: SystemTime::now(),
            expire_milli,
        }
    }

    pub fn has_expired(&self) -> Result<bool, Error> {
        Ok(self.expire_milli.is_some()
            && SystemTime::now()
                .duration_since(self.last_update_time)?
                .as_millis()
                > self.expire_milli.unwrap())
    }
}

#[derive(Debug)]
pub struct AppData {
    /// 40 character alphanumeric string.
    pub listening_port: u32,
    pub master: Option<AppDataMaster>,
    pub replica: Option<AppDataReplication>,
}

impl AppData {
    pub fn new_master(listening_port: u32) -> Result<Self, Error> {
        Ok(AppData {
            listening_port,
            master: Some(AppDataMaster {
                // replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_owned(),
                replid: pseudo_random_ascii_alphanumeric(40)?,
                repl_offset: 0,
            }),
            replica: None,
        })
    }

    pub fn new_replica(listening_port: u32, replica_config: AppDataReplication) -> Self {
        AppData {
            listening_port,
            master: None,
            replica: Some(replica_config),
        }
    }
}

#[derive(Debug)]
pub struct AppDataReplication {
    pub master_host: String,
    pub master_port: u32,
}

impl Clone for AppDataReplication {
    fn clone(&self) -> Self {
        AppDataReplication {
            master_host: self.master_host.clone(),
            master_port: self.master_port.clone(),
        }
    }
}

#[derive(Debug)]
pub struct AppDataMaster {
    pub replid: String,
    pub repl_offset: u32,
}

#[cfg(test)]
mod tests {
    use super::InMemoryRecord;

    use std::{thread, time::Duration};

    #[test]
    fn has_expired_passes() -> Result<(), anyhow::Error> {
        let expires = InMemoryRecord::new("".to_owned(), Some(1));
        let does_not_expire = InMemoryRecord::new("".to_owned(), Some(3));

        thread::sleep(Duration::from_millis(2));

        assert_eq!(expires.has_expired()?, true);
        assert_eq!(does_not_expire.has_expired()?, false);

        Ok(())
    }
}
