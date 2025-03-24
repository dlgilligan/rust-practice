mod api;
mod model;
mod queue;
mod repository;

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use api::task::{complete_task, fail_task, get_task, pause_task, start_task, submit_task};
use log::info;
use queue::redis::RedisQueue;
use repository::mongodb::MongoRepository;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logging, can use log macros after this
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    // Initialize MongoDB Repository
    let mongo_repo = match MongoRepository::init().await {
        Ok(repo) => {
            info!("MongoDB repository initialized");
            repo
        }
        Err(e) => {
            panic!("Failed to initialize MongoDB repository: {:?}", e);
        }
    };

    // Initialize Redis Queue
    let redis_queue = match RedisQueue::init() {
        Ok(queue) => {
            info!("Redis queue initialized");
            queue
        }
        Err(e) => {
            panic!("Failed to initialize Redis queue: {:?}", e);
        }
    };

    // Pass in closure that sets up everything for the web application
    // Closure is ran everytime actix starts a new thread
    HttpServer::new(move || {
        let logger = Logger::default();

        // Create shared app data for this thread
        let mongo_data = Data::new(mongo_repo.clone());
        let redis_data = Data::new(redis_queue.clone());

        App::new()
            .wrap(logger)
            .app_data(mongo_data) // Shared MongoDB repository
            .app_data(redis_data) // Shared Redis queue
            .service(get_task)
            .service(submit_task)
            .service(start_task)
            .service(complete_task)
            .service(pause_task)
            .service(fail_task)
    })
    .bind(("0.0.0.0", 80))? // Bind to all interfaces to work in Docker
    .run()
    .await
}
