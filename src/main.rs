#![feature(proc_macro_hygiene, decl_macro)]
#![feature(bind_by_move_pattern_guards)]

#[macro_use]
extern crate rocket;

mod project;
mod routes;
mod state;
mod walk_pandoc;

use std::path::PathBuf;

use rocket::config::{Config, Environment};
use rocket_cors::CorsOptions;

fn cors_options() -> CorsOptions {
    Default::default()
}

fn main() {
    rocket::custom(
        Config::build(Environment::Staging)
            .address("0.0.0.0")
            .unwrap(),
    )
    .manage(state::ProjectsState::new(PathBuf::from("./new_data/")))
    .mount(
        "/api2",
        routes![
            routes::get_editor_id,
            routes::projects,
            routes::new_project,
            routes::project_routes::new_file,
            routes::project_routes::index,
            routes::project_routes::index_delta,
            routes::project_routes::file_src,
            routes::project_routes::file_compiled,
            routes::project_routes::edit_src,
            routes::project_routes::static_files,
        ],
    )
    .mount("/", rocket_contrib::serve::StaticFiles::from("./app/dist/"))
    .attach(cors_options().to_cors().expect("To not fail"))
    .launch();
}
