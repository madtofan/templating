use clap::Parser;

#[derive(Parser)]
pub struct AppConfig {
    #[arg(long, env)]
    pub database_url: String,
    #[arg(long, env)]
    pub rust_log: String,
    #[arg(long, env)]
    pub service_url: String,
    #[arg(long, env)]
    pub service_port: u32,
    #[arg(long, env)]
    pub run_migrations: bool,
    #[arg(long, env)]
    pub seed: bool,
}
