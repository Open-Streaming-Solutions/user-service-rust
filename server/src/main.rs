use chrono::Local;
use user_service_rpc::rpc::user_service_server::UserServiceServer;
use dotenv::dotenv;
use tonic::transport::Server;
use log::{info};
use fern;
use fern::colors::{Color, ColoredLevelConfig};
use clap::Parser;

#[derive(Parser,)]
#[clap(author, version, about = "Типо сервер")]
struct Args {
    #[arg(short,long)]
    port: usize
}


mod app;
use app::services::UserServiceCore;
mod adapters;

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


    let addr = format!("127.0.0.1:{}",args.port);
    let user_service = UserServiceCore::default();

    info!("UserServiceServer listening on {}", addr);

    Server::builder()
        .add_service(UserServiceServer::new(user_service))
        .serve(addr.parse().unwrap())
        .await?;

    Ok(())
}
