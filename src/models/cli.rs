use super::db::app_data::AppDataReplication;

#[derive(Debug)]
pub struct AppCliArgs {
    pub port: u16,
    pub replica_of: Option<CliArgsReplication>,
}

#[derive(Debug)]
pub struct CliArgsReplication {
    pub master_host: String,
    pub master_port: u16,
}

impl Into<AppDataReplication> for CliArgsReplication {
    fn into(self) -> AppDataReplication {
        AppDataReplication {
            master_host: self.master_host,
            master_port: self.master_port,
        }
    }
}

pub struct AppCliFlagName {}

impl AppCliFlagName {
    pub const PORT: &'static str = "--port";
    pub const PORT_SHORT: &'static str = "-p";

    pub const REPLICA_OF: &'static str = "--replicaof";
}
