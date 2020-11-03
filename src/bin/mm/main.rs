use structopt::StructOpt;
use micromanager::Cmd;

#[derive(Debug, StructOpt)]
struct MMOpts {
    #[structopt(subcommand)]
    cmd: Cmd,    
    #[structopt(default_value = "http://localhost:8881", long = "server")]    
    host: String
}

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let options: MMOpts = MMOpts::from_args();
    let client = reqwest::Client::new();
    let res = client.post(&options.host)
    	.json(&options.cmd)
    	.send().await?;
    let body = res.text().await?;
    println!("{}", body); 
    Ok(())
}
