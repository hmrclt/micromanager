use actix_web::{get, post, App, HttpResponse, Result, HttpServer, Responder, web};
use micromanager::Cmd;
use std::sync::Mutex;

pub struct ApplicationState {
    running: Vec<String>
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/")]
async fn cmd(state: web::Data<Mutex<ApplicationState>>, request: web::Json<Cmd>) -> Result<String> {

    match request {
	web::Json(v) => match v {
	    Cmd::Start{service_name} => {
		let mut state_l = state.lock().unwrap();
		state_l.running.push(service_name.clone());
		let message = format!("Starting {} {:?}", service_name, state_l.running);
		println!("{}", message);
		Ok(message)
	    },
	    Cmd::Stop{service_name} => {
		let mut state_l = state.lock().unwrap();
		let index = state_l.running.iter().position(|x| *x == service_name).expect("no such service"); // TODO handle exception
		state_l.running.remove(index);
		let message = format!("Stop {} {:?}", service_name, state_l.running);
		println!("{}", message);
		Ok(message)
 	    },
	    Cmd::Status => Ok(format!("Status"))
	}
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(Mutex::new(ApplicationState{running: Vec::new()}));
    HttpServer::new(move || {
        App::new()
	    .app_data(state.clone())
            .service(hello).service(cmd)
    })
    .bind("127.0.0.1:8881")?
    .run()
    .await
}
