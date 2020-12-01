use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use micromanager::Cmd;
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

                // TODO: Handle process already running
                let new_process: Child = Command::new("grep")
                    .arg(&service_name)
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

    let config = fs::read_to_string(format!(
        "{}/service-manager-config/services.json",
        sm_workspace
    ))?;

    println!("{}", config);

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
