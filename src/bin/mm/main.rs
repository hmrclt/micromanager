extern crate rs_docker;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
enum Cmd {
    Start {
	service_name: String
    },
    Stop {
	service_name: String
    },
    Status 
}

fn main() {
    let options: Cmd = Cmd::from_args();
    println!("Hello cli!");

    match options {
	Cmd::Start {service_name} =>
	    println!("starting service {}", service_name),
	Cmd::Stop {service_name} =>
	    println!("stopping service {}", service_name),
	Cmd::Status =>
	    println!("getting status"),
    };
}
