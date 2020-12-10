use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub enum Cmd {
    Start { service_name: String },
    Stop { service_name: String },
    Status,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceManagerSources {
    repo: String
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceManagerBinary {
    artifact: String,
    #[serde(rename = "groupId")] group_id: String,
    cmd: Vec<String>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServiceManagerConfig {
    name: String,
    template: Option<String>,
    #[serde(rename = "defaultPort")] default_port: Option<u16>,
    sources: Option<ServiceManagerSources>,
    binary: Option<ServiceManagerBinary>
}
