pub mod schema;

use uuid::Uuid;
use crate::app::structs::User;
use crate::adapters::UserRepository;
use async_trait::async_trait;
use diesel::{OptionalExtension, PgConnection, QueryDsl, r2d2, RunQueryDsl, ExpressionMethods};
use diesel::r2d2::{ConnectionManager, PooledConnection};

/*
Читается так:
Определение алиаса Pool для библиотечного типа Pool, Который содершит структуру Для подключения к БД postgresql
*/
type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbRepository {
    pool: Pool,
}

impl DbRepository {
    pub fn new() -> Self {
        let database_url = "postgres://postgres:Qwe12345@localhost/user-service-database";
        let manager = ConnectionManager::<PgConnection>::new(database_url);
        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        DbRepository { pool }
    }

    fn get_conn(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.get().expect("Failed to get a connection")
    }
}


#[async_trait]
impl UserRepository for DbRepository {
    async fn add_user(&self, user: User) {
        use self::schema::users;

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
        use self::schema::users::dsl::*;

        let conn = &mut self.get_conn();
        users.load::<User>(conn)
            .expect("Error loading users")
    }

    async fn get_user(&self, user_id: &Uuid) -> Option<User> {
        use self::schema::users::dsl::*;

        let conn = &mut self.get_conn();
        users.filter(id.eq(user_id))
            .first::<User>(conn)
            .optional()
            .expect("Error loading user")
    }

    async fn get_user_id(&self, user_id: &Uuid) -> Option<Uuid> {
        self.get_user(user_id).await.map(|user| user.id)
    }

    async fn get_user_id_by_nickname(&self, nickname: &str) -> Option<Uuid> {
        use self::schema::users::dsl::*;

        let conn = &mut self.get_conn();
        users.filter(user_name.eq(nickname))
            .select(id)
            .first::<Uuid>(conn)
            .optional()
            .expect("Error loading user ID by nickname")
    }

    async fn update_user_by_id(&self, user_id: &Uuid, updated_user: User) -> Option<()> {
        use self::schema::users::dsl::*;

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
        use self::schema::users::dsl::*;

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
