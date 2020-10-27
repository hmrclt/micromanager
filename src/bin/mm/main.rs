use structopt::StructOpt;
use serde::Serialize;
use reqwest::{Error,Response};

#[derive(Debug, StructOpt, Serialize)]
enum Cmd {
    Start {
	service_name: String
    },
    Stop {
	service_name: String
    },
    Status 
}

#[derive(Debug, StructOpt)]
struct MMOpts {
    #[structopt(subcommand)]
    cmd: Cmd,    
    #[structopt(default_value = "http://localhost:8881")]    
    host: String
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let options: MMOpts = MMOpts::from_args();
    println!("Hello cli!");

    let client = reqwest::Client::new();
    let payload = serde_json::to_string(&options.cmd).unwrap();
    let res = client.post(&options.host)
    	.body(payload)
    	.send().await?;
 
    let body = res.text().await?;
    println!("Body:\n{}", body);
 
    Ok(())
}
