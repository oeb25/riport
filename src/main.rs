#![feature(async_await)]

use actix::*;
use actix_files as fs;
use actix_web::{get, web, App, Error, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use futures::future::{ok, Future};
use listenfd::ListenFd;

use std::path::PathBuf;

mod c2s;
mod client;
mod s2c;

mod doc;
mod file;
mod hub;
mod project;
mod walk_pandoc;

use crate::client::Client;
use crate::file::FileId;
use crate::hub::Hub;
use crate::project::ProjectId;

#[get("/ws/")]
fn start_websocket(
    req: HttpRequest,
    stream: web::Payload,
    hub: web::Data<Addr<Hub>>,
) -> impl Responder {
    let resp = ws::start(Client::new(hub.get_ref().clone()), &req, stream);
    resp
}

// #[get("/artifacts/{project_id}/{file_id}/{rest:.*}")]
fn compile_artifact(
    info: web::Path<(u64, u64, PathBuf)>,
    hub: web::Data<Addr<Hub>>,
) -> Box<dyn Future<Item = fs::NamedFile, Error = Error>> {
    let (project_id, file_id, rest) = info.into_inner();
    let project_id = ProjectId { project_id };
    let file_id = FileId { file_id };

    let req = hub::GetCompileArtifactDir {
        project_id,
        file_id,
    };

    println!("{:?}", req);

    Box::new(
        hub.send(req)
            .map_err::<_, Error>(|e| {
                println!("err");
                e.into()
            })
            .then(move |res| {
                println!("ehh");
                let p = res?.path.join(rest);
                println!("Reading file at {:?}", p);
                fs::NamedFile::open(p).map_err(|e| e.into())
            }),
    )
}
fn main() {
    let mut listenfd = ListenFd::from_env();

    let sys = System::new("my-system");

    let hub = Hub::new(PathBuf::from("./tmp")).unwrap().start();

    let server = HttpServer::new(move || {
        App::new()
            .data(hub.clone())
            .service(start_websocket)
            .route(
                "/artifacts/{project_id}/{file_id}/{rest:.*}",
                web::to_async(compile_artifact),
            )
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
