use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{r2d2, PgConnection, sql_query, RunQueryDsl};
use diesel_migrations::{MigrationHarness, EmbeddedMigrations, embed_migrations};
use log::{info, debug, error};

use crate::errors::{DbError, MigrationError};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/// Определение алиаса Pool для библиотечного типа Pool, который принимает структуру для подключения к БД PostgreSQL
pub(crate) type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbRepository {
    pub(crate) pool: Pool,
}

impl DbRepository {
    pub fn new(database_url: String) -> Self {
        debug!("Creating new DbRepository with database URL: {}", database_url);
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        let repo = DbRepository { pool };

        // Применение миграций при создании нового репозитория
        repo.manage_migration().expect("Failed to run migrations");

        repo
    }

    pub(crate) fn get_conn(&self) -> Result<PooledConnection<ConnectionManager<PgConnection>>, DbError> {
        debug!("Attempting to get a connection from the pool");
        match self.pool.get() {
            Ok(conn) => {
                debug!("Successfully obtained a connection from the pool");
                Ok(conn)
            },
            Err(e) => {
                debug!("Failed to obtain a connection from the pool: {}", e);
                Err(DbError::ConnectionError(e.to_string()))
            }
        }
    }

    pub fn manage_migration(&self) -> Result<(), MigrationError> {
        info!("Checking for pending migrations");
        debug!("Attempting to get a connection for checking migrations");
        let conn = &mut self.get_conn().map_err(|e| {
            error!("Failed to get a connection for checking migrations: {}", e);
            MigrationError::MigrationFailed(e.to_string())
        })?;

        debug!("Successfully obtained a connection for checking migrations");

        // Check if the database is initialized
        let is_initialized = sql_query("SELECT 1 FROM information_schema.tables WHERE table_name = 'users'")
            .execute(conn)
            .is_ok();

        if !is_initialized {
            info!("Database is not initialized. Running initial setup migrations.");
            conn.run_pending_migrations(MIGRATIONS).map_err(|e| {
                error!("Failed to run initial setup migrations: {}", e);
                MigrationError::MigrationFailed(e.to_string())
            })?;
            info!("Initial setup migrations complete");
        } else {
            let pending_migrations = conn.pending_migrations(MIGRATIONS).map_err(|e| {
                error!("Failed to check for pending migrations: {}", e);
                MigrationError::MigrationFailed(e.to_string())
            })?;

            if pending_migrations.is_empty() {
                info!("No pending migrations found");
                return Ok(());
            }

            info!("Running pending migrations");
            let applied_migrations = conn.run_pending_migrations(MIGRATIONS).map_err(|e| {
                error!("Failed to run migrations: {}", e);
                MigrationError::MigrationFailed(e.to_string())
            })?;

            for migration in applied_migrations {
                info!("Applied migration: {}", migration);
            }

            info!("Migrations complete");
        }

        Ok(())
    }
}