#![feature(async_await)]

use actix::*;
use actix_files as fs;
use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use listenfd::ListenFd;

mod client;
mod doc;
mod file;
mod hub;
mod project;
mod walk_pandoc;

use crate::client::Client;
use crate::hub::Hub;

fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

fn index2() -> impl Responder {
    HttpResponse::Ok().body("Hello world again!")
}

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

// enum Msg {
//     Connect { addr: Recipient<ServerMsg> },
//     Disconnect { id: usize },
//     Message { contents: String },
// }

// impl Message for Msg {
//     type Result = MsgResponse;
// }

// #[derive(MessageResponse)]
// enum MsgResponse {
//     None,
//     AssignID { id: usize },
// }

// enum ServerMsg {
//     Message { contents: String },
// }

// impl Message for ServerMsg {
//     type Result = ();
// }

// #[derive(Default)]
// struct Hub {
//     connections: Vec<Option<Recipient<ServerMsg>>>,
//     projects: HashMap<ProjectId, Addr<Project>>,
// }

// impl Actor for Hub {
//     type Context = Context<Self>;
// }

// impl Handler<Msg> for Hub {
//     type Result = MsgResponse;

//     fn handle(&mut self, msg: Msg, ctx: &mut Context<Self>) -> MsgResponse {
//         println!("Hub got message");
//         match msg {
//             Msg::Connect { addr } => {
//                 let id = self.connections.len();
//                 println!("New connection: {}", id);
//                 self.connections.push(Some(addr));
//                 MsgResponse::AssignID { id }
//             }
//             Msg::Disconnect { id } => {
//                 println!("Disconnected: {}", id);
//                 self.connections[id] = None;
//                 MsgResponse::None
//             }
//             Msg::Message { contents } => {
//                 println!("Message recived: {}", contents);
//                 for con in self.connections.iter().filter_map(|x| x.as_ref()) {
//                     con.do_send(ServerMsg::Message {
//                         contents: contents.clone(),
//                     });
//                 }
//                 MsgResponse::None
//             }
//         }
//     }
// }

// #[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
// struct ProjectId(usize);

// struct Project {
//     id: ProjectId,
//     name: String,
// }

// impl Actor for Project {
//     type Context = Context<Self>;
// }

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
        server.bind("0.0.0.0:8080").unwrap()
    };

    server.start();

    sys.run().unwrap();
}
