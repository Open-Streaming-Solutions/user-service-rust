use std::env;

#[derive(Debug)]
pub struct Config {
    pub database_url: String,
    pub server_addr: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
        let server_host = env::var("SERVER_HOST").expect("SERVER_HOST must be set");
        let server_addr = format!("{}:{}", server_host, server_port);

        Config {
            database_url,
            server_addr,
        }
    }
}
