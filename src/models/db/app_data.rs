use std::{collections::HashMap, sync::Arc};

use anyhow::Error;
use tokio::sync::Mutex;

use crate::{models::t_stream::TStream, utils::pseudo_random_ascii_alphanumeric};

#[derive(Debug)]
pub struct AppData {
    pub listening_port: u16,
    master: Option<AppDataMaster>,
    replication: Option<AppDataReplication>,
}

impl AppData {
    pub fn new_master(listening_port: u16) -> Result<Self, Error> {
        Ok(AppData {
            listening_port,
            master: Some(AppDataMaster {
                // replid: "8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb".to_owned(),
                replid: pseudo_random_ascii_alphanumeric(40)?,
                repl_offset: 0,
                slaves: HashMap::new(),
            }),
            replication: None,
        })
    }

    pub fn new_replica(listening_port: u16, replica_config: AppDataReplication) -> Self {
        AppData {
            listening_port,
            master: None,
            replication: Some(replica_config),
        }
    }

    pub fn get_master_data_ref(&self) -> Option<&AppDataMaster> {
        self.master.as_ref()
    }

    pub fn get_master_data_mut(&mut self) -> Option<&mut AppDataMaster> {
        self.master.as_mut()
    }

    pub fn get_replication_data_ref(&self) -> Option<&AppDataReplication> {
        self.replication.as_ref()
    }
}

#[derive(Debug)]
pub struct AppDataMaster {
    /// 40 character alphanumeric string.
    pub replid: String,
    pub repl_offset: u32,
    pub slaves: HashMap<u16, AppDataSlave>,
}

#[derive(Debug)]
pub struct AppDataSlave {
    pub port: u16,
    pub tcp_stream: Arc<Mutex<dyn TStream>>,
    pub full_handshake: bool,
}

#[derive(Debug)]
pub struct AppDataReplication {
    pub master_host: String,
    pub master_port: u16,
}

impl Clone for AppDataReplication {
    fn clone(&self) -> Self {
        AppDataReplication {
            master_host: self.master_host.clone(),
            master_port: self.master_port.clone(),
        }
    }
}
