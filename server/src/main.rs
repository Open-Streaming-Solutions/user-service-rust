use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use lib_rpc::userpb::user_service_server::UserServiceServer;
use log::info;
use std::sync::Arc;
use tonic::transport::Server;

mod app;

use crate::adapters::postgres::DbRepository;
use crate::app::user_service::UserServiceCore;
use crate::config::Config;

mod adapters;
mod config;
mod errors;
mod repo;
mod types;

/*
#[derive(Parser)]
#[clap(author, version, about = "Типо сервер")]
struct Args {
    #[arg(short, long, default_value = "8080")]
    port: usize,
}
*/
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
    setup_logger()?;
    //let args = Args::parse();

    let config = Config::from_env();

    info!("Initializing the UserServiceServer...");

    ///Переделать
    let db_repository = DbRepository::new(config.database_url)
        .map_err(|e| {
            eprintln!("Failed to create DbRepository: {:?}", e);
            e
        })
        .unwrap();

    let user_service = UserServiceCore {
        repository: Arc::new(db_repository),
    };

    info!("UserServiceServer listening on {}", config.server_addr);

    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(config.server_addr.parse().unwrap())
        .await?;

    Ok(())
}
