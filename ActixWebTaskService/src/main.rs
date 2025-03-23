mod api;
mod model;
mod repository;
use api::task::get_task;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use repository::ddb::DDBRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging, can use log macros after this
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let config = aws_config::load_from_env().await;
    // Pass in closure that sets up everything for the web application
    // Closure is ran everytime actix starts a new thread
    HttpServer::new(move || {
        let ddb_repo::DDBRepository::init(
            String::from("task"),
            config.clone(), // Create a copy for every thread
        );
        let ddb_data = Data::new(ddb_repo); // To pass shared data, need to use data struct that
        // implements FromRequest trait
        let logger = Logger::default();
        App::new()
            .wrap(logger)
            .app_data(ddb_data) // Shared state that will be injected into handler functions
            .service(get_task)
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}
