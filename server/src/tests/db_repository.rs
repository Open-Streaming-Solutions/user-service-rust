use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use pretty_assertions::assert_eq;
use tokio;
use uuid::Uuid;

use crate::adapters::database::{DbRepository, Pool};
use crate::adapters::database::schema::users::dsl::users;
use crate::adapters::UserRepository;
use crate::app::structs::User;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

fn setup_test_db() -> Pool {
    let database_url = "postgres://postgres:Qwe12345@localhost/user-service-test";
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    // Выполнение миграций
    let conn = &mut pool.get().expect("Failed to get a connection");
    conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");

    pool
}

fn clear_test_db(pool: &Pool) {

    let conn = &mut pool.get().expect("Failed to get a connection");
    diesel::delete(users)
        .execute(conn)
        .expect("Failed to clear test database");
}
#[tokio::test]
async fn test_manage_migration() {
    let pool = setup_test_db();
    let repo = DbRepository { pool: pool.clone() };
    let result = repo.manage_migration();
    assert!(result.is_ok(), "Migration should run successfully");
}

#[tokio::test]
async fn add_user() {
    let pool = setup_test_db();
    let repo = DbRepository { pool: pool.clone() };
    clear_test_db(&pool);

    let user = User { id: Uuid::now_v7(), user_name: "testuser".to_string(), user_email: "testuser@test.com".to_string() };
    repo.add_user(user.clone()).await;

    let all_users = repo.get_all_users().await;
    assert_eq!(all_users.len(), 1);
    assert_eq!(all_users[0].user_name, "testuser");
}

#[tokio::test]
async fn get_user() {
    let pool = setup_test_db();
    let repo = DbRepository { pool: pool.clone() };
    clear_test_db(&pool);

    let user_id = Uuid::now_v7();
    let user = User { id: user_id, user_name: "testuser".to_string(), user_email: "testuser@test.com".to_string() };
    repo.add_user(user.clone()).await;

    let fetched_user = repo.get_user(&user_id).await;
    assert!(fetched_user.is_some());
    assert_eq!(fetched_user.unwrap().user_name, "testuser");
}

#[tokio::test]
async fn get_user_id() {
    let pool = setup_test_db();
    let repo = DbRepository { pool: pool.clone() };
    clear_test_db(&pool);

    let user_id = Uuid::now_v7();
    let user = User { id: user_id, user_name: "testuser".to_string(), user_email: "testuser@test.com".to_string() };
    repo.add_user(user.clone()).await;

    let fetched_user_id = repo.get_user_id(&user_id).await;
    assert!(fetched_user_id.is_some());
    assert_eq!(fetched_user_id.unwrap(), user_id);
    clear_test_db(&pool);
}

#[tokio::test]
async fn get_user_id_by_nickname() {
    let pool = setup_test_db();
    let repo = DbRepository { pool: pool.clone() };
    clear_test_db(&pool);

    let user_id = Uuid::now_v7();
    let user = User { id: user_id, user_name: "testuser".to_string(), user_email: "testuser@test.com".to_string() };
    repo.add_user(user.clone()).await;

    let fetched_user_id = repo.get_user_id_by_nickname("testuser").await;
    assert!(fetched_user_id.is_some());
    assert_eq!(fetched_user_id.unwrap(), user_id);
    clear_test_db(&pool);
}

#[tokio::test]
async fn update_user_by_id() {
    let pool = setup_test_db();
    let repo = DbRepository { pool: pool.clone() };
    clear_test_db(&pool);

    let user_id = Uuid::now_v7();
    let user = User { id: user_id, user_name: "testuser".to_string(), user_email: "testuser@test.com".to_string() };
    repo.add_user(user.clone()).await;

    let updated_user = User { id: user_id, user_name: "updateduser".to_string(), user_email: "updateduser@test.com".to_string() };
    let result = repo.update_user_by_id(&user_id, updated_user.clone()).await;
    assert!(result.is_some());

    let fetched_user = repo.get_user(&user_id).await;
    assert!(fetched_user.is_some());
    assert_eq!(fetched_user.unwrap().user_name, "updateduser");
    clear_test_db(&pool);
}

#[tokio::test]
async fn update_user_by_nickname() {
    let pool = setup_test_db();
    let repo = DbRepository { pool: pool.clone() };
    clear_test_db(&pool);

    let user_id = Uuid::now_v7();
    let user = User { id: user_id, user_name: "testuser".to_string(), user_email: "testuser@test.com".to_string() };
    repo.add_user(user.clone()).await;

    let updated_user = User { id: user_id, user_name: "updateduser".to_string(), user_email: "updateduser@test.com".to_string() };
    let result = repo.update_user_by_nickname("testuser", updated_user.clone()).await;
    assert!(result.is_some());

    let fetched_user = repo.get_user(&user_id).await;
    assert!(fetched_user.is_some());
    assert_eq!(fetched_user.unwrap().user_name, "updateduser");
    clear_test_db(&pool);
}