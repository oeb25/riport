#![feature(async_await)]

use actix::*;
use actix_files as fs;
use actix_web::{get, web, App, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use listenfd::ListenFd;

mod c2s;
mod client;
mod s2c;

mod doc;
mod file;
mod hub;
mod project;
mod walk_pandoc;

use crate::client::Client;
use crate::hub::Hub;

#[get("/ws/")]
fn start_websocket(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<Addr<Hub>>,
) -> impl Responder {
    let resp = ws::start(Client::new(data.get_ref().clone()), &req, stream);
    println!("{:?}", resp);
    resp
}
fn main() {
    let mut listenfd = ListenFd::from_env();

    let sys = System::new("my-system");

    let hub = Hub::default().start();

    let server = HttpServer::new(move || {
        App::new()
            .data(hub.clone())
            .service(start_websocket)
            .service(fs::Files::new("/", "./frontend/dist"))
    });

    let server = if let Some(l) = listenfd.take_tcp_listener(0).unwrap() {
        server.listen(l).unwrap()
    } else {
        server.bind("0.0.0.0:8000").unwrap()
    };

    server.start();

    sys.run().unwrap();
}
