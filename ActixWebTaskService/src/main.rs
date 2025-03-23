mod api;
use api::task::get_task;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging, can use log macros after this
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // Pass in closure that sets up everything for the web application
    // Closure is ran everytime actix starts a new thread
    HttpServer::new(move || {
        let logger = Logger::default();
        App::new().wrap(logger).service(get_task)
    })
    .bind(("127.0.0.1", 80))?
    .run()
    .await
}
