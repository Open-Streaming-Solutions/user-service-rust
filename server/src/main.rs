use chrono::Local;
use fern::colors::{Color, ColoredLevelConfig};
use lib_rpc::userpb::user_service_server::UserServiceServer;
use log::info;
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
fn setup_logger(log_level: &str) -> Result<(), fern::InitError> {
    let log_level = match log_level.to_lowercase().as_str() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Debug, // Уровень по умолчанию
    };

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
        .level(log_level)
        .chain(std::io::stdout())
        .apply()?;

    Ok(())
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    setup_logger(config.get_log_level()).expect("Logger initialization failed");

    info!("Initializing the UserServiceServer...");

    //Переделать
    let db_repository = DbRepository::new(config.get_database_url())
        .map_err(|e| {
            eprintln!("Failed to create DbRepository: {:?}", e);
            e
        })
        .unwrap();

    let user_service = UserServiceCore::new(db_repository.into()).await;
    //ToDo Сделать обработку ошибок
    info!("UserServiceServer listening on {}", config.get_server_addr());
    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(config.get_server_addr().parse().unwrap())
        .await.expect("Initializing Server failed");


}
