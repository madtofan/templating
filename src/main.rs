use std::sync::Arc;

use crate::config::AppConfig;
use crate::handler::templating::RequestHandler;
use crate::repository::input::{DynInputRepositoryTrait, InputRepository};
use crate::repository::template::{DynTemplateRepositoryTrait, TemplateRepository};
use crate::seed::SeedService;
use crate::service::templating::{DynTemplatingServiceTrait, TemplatingService};
use clap::Parser;
use dotenv::dotenv;
use madtofan_microservice_common::{
    repository::connection_pool::ServiceConnectionManager,
    templating::templating_server::TemplatingServer,
};
use tonic::transport::Server;
use tracing::error;
use tracing::log::info;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod config;
mod handler;
mod repository;
mod seed;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let config = Arc::new(AppConfig::parse());

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.rust_log))
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Environment loaded and configuration parsed, initializing Postgres connection...");
    let pg_pool = ServiceConnectionManager::new_pool(&config.database_url)
        .await
        .expect("could not initialize the database connection pool");

    if config.run_migrations {
        info!("migrations enabled, running...");
        sqlx::migrate!()
            .run(&pg_pool)
            .await
            .unwrap_or_else(|err| error!("There was an error during migration: {:?}", err));
    }
    info!("Database configured! initializing repositories...");

    let app_host = &config.service_url;
    let app_port = &config.service_port;
    let app_url = format!("{}:{}", app_host, app_port).parse().unwrap();
    let inputs_repository =
        Arc::new(InputRepository::new(pg_pool.clone())) as DynInputRepositoryTrait;
    let template_repository = Arc::new(TemplateRepository::new(pg_pool, inputs_repository.clone()))
        as DynTemplateRepositoryTrait;

    info!("Repositories initialized, Initializing Services");
    let templating_service = Arc::new(TemplatingService::new(
        template_repository.clone(),
        inputs_repository.clone(),
    )) as DynTemplatingServiceTrait;

    info!("Services initialized, Initializing Handler");
    let request_handler = RequestHandler::new(templating_service.clone());

    if config.seed {
        info!("seeding enabled, creating test data...");
        SeedService::new(templating_service, template_repository)
            .seed()
            .await
            .expect("unexpected error occurred while seeding application data");
    }

    info!("Service ready for request at {:#?}!", app_url);
    Server::builder()
        .add_service(TemplatingServer::new(request_handler))
        .serve(app_url)
        .await?;
    Ok(())
}
