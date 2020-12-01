use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub enum Cmd {
    Start { service_name: String },
    Stop { service_name: String },
    Status,
}
