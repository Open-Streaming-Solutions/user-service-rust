use std::env;

#[derive(Debug)]
pub struct Config {
    database_url: String,
    server_addr: String,
    log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();

        let database_url = if cfg!(debug_assertions) {
            env::var("TEST_DATABASE_URL").unwrap_or_else(|_| "Test_database-url".to_string())
        } else {
            env::var("DATABASE_URL").expect("DATABASE_URL must be set")
        };
        let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8080".to_string());
        let server_host = env::var("SERVER_HOST").expect("SERVER_HOST must be set");
        let server_addr = format!("{}:{}", server_host, server_port);
        let log_level = env::var("LOG_LEVEL").expect("LOG_LEVEL must be set");

        Config {
            database_url,
            server_addr,
            log_level,
        }
    }
    pub fn get_database_url(&self) -> &str {
        &self.database_url
    }
    pub fn get_server_addr(&self) -> &str {
        &self.server_addr
    }

    pub fn get_log_level(&self) -> &str {
        &self.log_level
    }
}
