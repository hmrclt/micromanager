use structopt::StructOpt;
use serde::{Serialize, Deserialize};

#[derive(Debug, StructOpt, Serialize, Deserialize)]
pub enum Cmd {
    Start {
	service_name: String
    },
    Stop {
	service_name: String
    },
    Status 
}
