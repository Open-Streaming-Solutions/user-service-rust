use crate::adapters::postgres::DbRepository;
use crate::adapters::schema::users::dsl::users;
use crate::adapters::schema::users::{email, id, name};
use crate::repo::UserRepository;
use crate::types::User;
use async_trait::async_trait;
use diesel::associations::HasTable;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use uuid::Uuid;

#[async_trait]
impl UserRepository for DbRepository {
    async fn add_user(&self, user: User) {
        let conn = &mut self.get_conn();
        let new_user = User {
            id: user.id,
            name: user.name,
            email: user.email,
        };
        diesel::insert_into(users::table())
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
            .filter(name.eq(nickname))
            .select(id)
            .first::<Uuid>(conn)
            .optional()
            .expect("Error loading user ID by nickname")
    }

    async fn update_user_by_id(&self, user_id: &Uuid, updated_user: User) -> Option<()> {
        let conn = &mut self.get_conn();
        let target = users.filter(id.eq(user_id));
        let updated_rows = diesel::update(target)
            .set((name.eq(updated_user.name), email.eq(updated_user.email)))
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
        let target = users.filter(name.eq(nick_name));
        let updated_rows = diesel::update(target)
            .set((name.eq(updated_user.name), email.eq(updated_user.email)))
            .execute(conn)
            .expect("Error updating user by nickname");

        if updated_rows > 0 {
            Some(())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::adapters::postgres::{DbRepository, Pool};
    use crate::adapters::schema::users::dsl::users;
    use crate::repo::UserRepository;
    use crate::types::User;
    use diesel::prelude::*;
    use diesel::r2d2::{self, ConnectionManager};
    use dotenv::dotenv;
    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use std::env;
    use tokio;
    use uuid::Uuid;

    fn setup_test_db() -> r2d2::Pool<ConnectionManager<PgConnection>> {
        dotenv().ok();

        let database_url = env::var("TEST_DATABASE_URL").expect("DATABASE_URL must be set");
        let db_repo = DbRepository::new(database_url);

        db_repo
            .manage_migration()
            .expect("Failed to run migrations");

        db_repo.pool
    }

    fn clear_test_db(pool: &Pool) {
        let conn = &mut pool.get().expect("Failed to get a connection");
        diesel::delete(users)
            .execute(conn)
            .expect("Failed to clear test database");
    }

    #[tokio::test]
    #[serial]
    async fn test_manage_migration() {
        let pool = setup_test_db();
        let repo = DbRepository { pool: pool.clone() };
        let result = repo.manage_migration();
        assert!(result.is_ok(), "Migration should run successfully");
    }

    #[tokio::test]
    #[serial]
    async fn add_user() {
        let pool = setup_test_db();
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user = User {
            id: Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap(),
            name: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        repo.add_user(user.clone()).await;

        let all_users = repo.get_all_users().await;
        assert_eq!(all_users.len(), 1);
        assert_eq!(all_users[0].name, "testuser");
    }

    #[tokio::test]
    #[serial]
    async fn get_user_data_by_id() {
        let pool = setup_test_db();
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            name: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        repo.add_user(user.clone()).await;

        let fetched_user = repo.get_user(&user_id).await;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().name, "testuser");
    }

    #[tokio::test]
    #[serial]
    async fn get_user_id() {
        let pool = setup_test_db();
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            name: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        repo.add_user(user.clone()).await;

        let fetched_user_id = repo.get_user_id(&user_id).await;
        assert!(fetched_user_id.is_some());
        assert_eq!(fetched_user_id.unwrap(), user_id);
    }

    #[tokio::test]
    #[serial]
    async fn get_user_id_by_nickname() {
        let pool = setup_test_db();
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            name: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        repo.add_user(user.clone()).await;

        let fetched_user_id = repo.get_user_id_by_nickname("testuser").await;
        assert!(fetched_user_id.is_some());
        assert_eq!(fetched_user_id.unwrap(), user_id);
    }

    #[tokio::test]
    #[serial]
    async fn update_user_by_id() {
        let pool = setup_test_db();
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            name: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        repo.add_user(user.clone()).await;

        let updated_user = User {
            id: user_id,
            name: "updateduser".to_string(),
            email: "updateduser@test.com".to_string(),
        };
        let result = repo.update_user_by_id(&user_id, updated_user.clone()).await;
        assert!(result.is_some());

        let fetched_user = repo.get_user(&user_id).await;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().name, "updateduser");
    }

    #[tokio::test]
    #[serial]
    async fn update_user_by_nickname() {
        let pool = setup_test_db();
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            name: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        repo.add_user(user.clone()).await;

        let updated_user = User {
            id: user_id,
            name: "updateduser".to_string(),
            email: "updateduser@test.com".to_string(),
        };
        let result = repo
            .update_user_by_nickname("testuser", updated_user.clone())
            .await;
        assert!(result.is_some());

        let fetched_user = repo.get_user(&user_id).await;
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().name, "updateduser");
    }
}
