use chrono::Local;
use clap::Parser;
use dotenv::dotenv;
use fern::colors::{Color, ColoredLevelConfig};
use log::info;
use std::env;
use std::sync::Arc;
use tonic::transport::Server;
use lib_rpc::rpc::user_service_server::UserServiceServer;

mod app;

use crate::adapters::postgres::DbRepository;
use crate::app::user_service::UserServiceCore;

mod adapters;
mod types;
pub mod internal_repo;
mod config;

#[derive(Parser)]
#[clap(author, version, about = "Типо сервер")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: usize,
}

fn setup_logger() -> Result<(), fern::InitError> {
    let colors = ColoredLevelConfig::new()
        .trace(Color::White)
        .debug(Color::White)
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Cyan);
    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{}] {} <{}> {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.target(),
                colors.color(record.level()),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    setup_logger()?;
    let args = Args::parse();

    info!("Initializing the UserServiceServer...");

    //Переделать под хостнейм
    let addr = format!("0.0.0.0:{}", args.port);
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_repository = DbRepository::new(database_url);
    DbRepository::manage_migration(&db_repository).expect("Pizda");
    let user_service = UserServiceCore {
        repository: Arc::new(db_repository),
    };

    info!("UserServiceServer listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(addr.parse().unwrap())
        .await?;

    Ok(())
}
