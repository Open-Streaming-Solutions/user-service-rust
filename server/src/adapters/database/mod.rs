use async_trait::async_trait;
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::{r2d2, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use log::info;
use uuid::Uuid;

pub mod schema;
use crate::adapters::UserRepository;
use crate::app::structs::User;

use self::schema::users;
use self::schema::users::dsl::*;

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
        DbRepository { pool }
    }

    fn get_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().expect("Failed to get a connection")
    }

    pub fn manage_migration(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!("Running migrations!");
        let conn = &mut self.get_conn();
        conn.run_pending_migrations(MIGRATIONS)?;
        Ok(())
    }
}

#[async_trait]
impl UserRepository for DbRepository {
    async fn add_user(&self, user: User) {
        let conn = &mut self.get_conn();
        let new_user = User {
            id: user.id,
            user_name: user.user_name,
            user_email: user.user_email,
        };
        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(conn)
            .expect("Error saving new user");
    }

    async fn get_all_users(&self) -> Vec<User> {
        let conn = &mut self.get_conn();
        users.load::<User>(conn).expect("Error loading users")
    }

    async fn get_user(&self, user_id: &Uuid) -> Option<User> {
        let conn = &mut self.get_conn();
        users
            .filter(id.eq(user_id))
            .first::<User>(conn)
            .optional()
            .expect("Error loading user")
    }

    //Пока норм, но надо передумать.
    async fn get_user_id(&self, user_id: &Uuid) -> Option<Uuid> {
        self.get_user(user_id).await.map(|user| user.id)
    }

    async fn get_user_id_by_nickname(&self, nickname: &str) -> Option<Uuid> {
        let conn = &mut self.get_conn();
        users
            .filter(user_name.eq(nickname))
            .select(id)
            .first::<Uuid>(conn)
            .optional()
            .expect("Error loading user ID by nickname")
    }

    async fn update_user_by_id(&self, user_id: &Uuid, updated_user: User) -> Option<()> {
        let conn = &mut self.get_conn();
        let target = users.filter(id.eq(user_id));
        let updated_rows = diesel::update(target)
            .set((
                user_name.eq(updated_user.user_name),
                user_email.eq(updated_user.user_email),
            ))
            .execute(conn)
            .expect("Error updating user");

        if updated_rows > 0 {
            Some(())
        } else {
            None
        }
    }

    async fn update_user_by_nickname(&self, nick_name: &str, updated_user: User) -> Option<()> {
        let conn = &mut self.get_conn();
        let target = users.filter(user_name.eq(nick_name));
        let updated_rows = diesel::update(target)
            .set((
                user_name.eq(updated_user.user_name),
                user_email.eq(updated_user.user_email),
            ))
            .execute(conn)
            .expect("Error updating user by nickname");

        if updated_rows > 0 {
            Some(())
        } else {
            None
        }
    }
}
