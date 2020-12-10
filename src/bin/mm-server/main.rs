use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use micromanager::{Cmd, ServiceManagerConfig};
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Mutex;

use nix::sys::signal::{self, Signal};
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use std::{env, fs};

pub struct ApplicationState {
    running: HashMap<String, Child>,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/")]
async fn cmd(state: web::Data<Mutex<ApplicationState>>, request: web::Json<Cmd>) -> Result<String> {
    match request {
        web::Json(v) => match v {
            Cmd::Start { service_name } => {
                let mut state_l = state.lock().unwrap();

		let version = "0.9.0";
		let artifact_name = format!("uk.gov.hmrc::{}:{}", service_name, version);
		
                // TODO: Handle process already running
                let new_process: Child = Command::new("coursier")
                    .arg("launch")
                    .arg("--java-opt").arg("-Dhttp.port=9876")
		    .arg("--fork=false") // if set port is ignored, if not set can't shut down process
                    .arg("-r").arg("https://artefacts.tax.service.gov.uk/artifactory/hmrc-releases/")
                    .arg(&artifact_name)
                    .spawn()
                    .expect("oh no!");
                let pid = new_process.id();

                state_l.running.insert(service_name.clone(), new_process);
                let message = format!("Starting {} {:?}", service_name, state_l.running);

                tokio::spawn(async move {
                    match waitpid(Pid::from_raw(pid as i32), None) {
                        Ok(status) => println!("[main] Child exited with status {:?}", status),
                        Err(err) => panic!("[main] waitpid() failed: {}", err),
                    }
                });

                println!("{}", message);
                Ok(message)
            }
            Cmd::Stop { service_name } => {
                let mut state_l = state.lock().unwrap();
                match state_l.running.get(&service_name) {
                    Some(child) => {
                        let pid = Pid::from_raw(child.id() as i32);
                        signal::kill(pid, Signal::SIGTERM).expect("unable to kill");
                        state_l.running.remove(&service_name);
                        Ok(format!("Shutting down {}", service_name))
                    }
                    None => Ok("Process not found".to_string()),
                }
            }
            Cmd::Status => {
                let state_l = state.lock().unwrap();
                let message = format!("Running {:?}", state_l.running);
                Ok(message)
            }
        },
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(Mutex::new(ApplicationState {
        running: HashMap::new(),
    }));

    let sm_workspace = env::var("WORKSPACE").expect("WORKSPACE is undefined");

    let config: String = fs::read_to_string(format!(
        "{}/service-manager-config/services.json",
        sm_workspace
    ))?;

    let config: HashMap<String, ServiceManagerConfig> = serde_json::from_str(&config).unwrap();
    
    println!("{:?}", config);

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(hello)
            .service(cmd)
    })
    .bind("127.0.0.1:8881")?
    .run()
    .await
}
