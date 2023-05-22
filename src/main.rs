use std::sync::Arc;

use crate::config::AppConfig;
use crate::handler::templating::RequestHandler;
use crate::repository::input::{DynInputRepositoryTrait, InputRepository};
use crate::repository::template::{DynTemplateRepositoryTrait, TemplateRepository};
use crate::seed::SeedService;
use crate::service::templating::{DynTemplatingServiceTrait, TemplatingService};
use crate::templating::templating_server::TemplatingServer;
use clap::Parser;
use common::repository::connection_pool::ServiceConnectionManager;
use dotenv::dotenv;
use tonic::transport::Server;
use tracing::log::info;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod handler;
mod repository;
mod seed;
mod service;
pub mod templating {
    tonic::include_proto!("templating");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().expect("Failed to read .env file, please add a .env file to the project root");

    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Environment loaded and configuration parsed, initializing Postgres connection and running migrations...");
    let pg_pool = ServiceConnectionManager::new_pool(&config.database_url)
        .await
        .expect("could not initialize the database connection pool");

    let app_host = &config.service_url;
    let app_port = &config.service_port;
    let app_url = format!("{}:{}", app_host, app_port).parse().unwrap();
    let inputs_repository =
        Arc::new(InputRepository::new(pg_pool.clone())) as DynInputRepositoryTrait;
    let template_repository = Arc::new(TemplateRepository::new(pg_pool, inputs_repository.clone()))
        as DynTemplateRepositoryTrait;
    let templating_service = Arc::new(TemplatingService::new(
        template_repository.clone(),
        inputs_repository.clone(),
    )) as DynTemplatingServiceTrait;
    let request_handler = RequestHandler::new(templating_service.clone());

    if config.seed {
        info!("seeding enabled, creating test data...");
        SeedService::new(templating_service, template_repository)
            .seed()
            .await
            .expect("unexpected error occurred while seeding application data");
    }

    info!("Service ready for request!");
    Server::builder()
        .add_service(TemplatingServer::new(request_handler))
        .serve(app_url)
        .await?;
    Ok(())
}
