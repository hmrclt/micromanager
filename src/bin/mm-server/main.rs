use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use micromanager::{Cmd, ServerConfig};
use std::collections::HashMap;
use std::process::{Child, Command};
use std::sync::Mutex;

use nix::sys::signal::{self, Signal};
use nix::sys::wait::waitpid;
use nix::unistd::Pid;
use bollard::Docker;
use bollard::container::{CreateContainerOptions, Config};
use hocon::HoconLoader;

pub struct ApplicationState {
    running: HashMap<String, Child>,
    db: sqlite::Connection,
    docker: Docker
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

		let artifact_name = format!("uk.gov.hmrc::{}", service_name); // assuming service_name is {service}:{version}
		
		let coursier_out = Command::new("coursier")
		    .arg("fetch")
		    .arg("-r").arg("https://artefacts.tax.service.gov.uk/artifactory/hmrc-releases/")
		    .arg(&artifact_name)
		    .output()
		    .expect("Unable to run coursier");
		
		let stdout = coursier_out.stdout;
		let stdout_string = String::from_utf8(stdout).unwrap();
		let lines: Vec<&str> = stdout_string.lines().collect();
		let jars = lines.join(":");

		// TODO: Handle process already running		
                let new_process: Child = Command::new("java")
		    .arg("-Dhttp.port=9876")
                    .arg("-cp")		    
                    .arg(jars)
		    .arg("play.core.server.ProdServerStart")
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
    
    let connection = sqlite::open(":memory:").expect("Unable to connect to database");
    
    let docker = Docker::connect_with_local_defaults().expect("unable to connect to docker");
    // let version = docker.version().await.expect("unable to get docker version");
    // println!("Docker version: {:?}", version);


    let options = Some(CreateContainerOptions{
	name: "my-new-container",
    });

    let config = Config {
	image: Some("debian"),
	cmd: Some(vec!["md5sum"]),
	..Default::default()
    };

    docker.create_container(options, config).await.unwrap();
    docker.remove_container("my-new-container", None).await.unwrap();
    
    let config: ServerConfig = HoconLoader::new()
        .load_file("micro-manager.conf").expect("Unable to access config file at micro-manager.conf")
        .resolve().unwrap();
    
    let state = web::Data::new(Mutex::new(ApplicationState {
        running: HashMap::new(),
	db: connection,
	docker
    }));

    // let sm_workspace = env::var("WORKSPACE").expect("WORKSPACE is undefined");

    // let config: String = fs::read_to_string(format!(
    //     "{}/service-manager-config/services.json",
    //     sm_workspace
    // ))?;

    // let config: HashMap<String, ServiceManagerConfig> = serde_json::from_str(&config).unwrap();

    let address = format!("{}:{}", config.host, config.port);
    let http = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(hello)
            .service(cmd)
    })
    .bind(&address)?.run();
    println!("Listening on {}", address);
    http.await
}
