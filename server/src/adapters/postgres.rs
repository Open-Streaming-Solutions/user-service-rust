use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{r2d2, PgConnection};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::info;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

/*
Читается так:
Определение алиаса Pool для библиотечного типа Pool, Который принимает структуру Для подключения к БД postgresql
*/
pub(crate) type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbRepository {
    pub(crate) pool: Pool,
}

impl DbRepository {
    pub fn new(database_url: String) -> Self {
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        let repo = DbRepository { pool };

        // Применение миграций при создании нового репозитория
        repo.manage_migration().expect("Failed to run migrations");

        repo
    }

    pub(crate) fn get_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().expect("Failed to get a connection")
    }

    pub fn manage_migration(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Running migrations!");
        let conn = &mut self.get_conn();
        conn.run_pending_migrations(MIGRATIONS)?;
        Ok(())
    }
}
