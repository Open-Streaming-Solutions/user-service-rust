use crate::adapters::postgres::DbRepository;
use crate::adapters::schema::users::dsl::users;
use crate::adapters::schema::users::{email, id, username};
use crate::errors::DbError;
use crate::repo::{RepoError, UserRepository};
use crate::types::User;
use async_trait::async_trait;
use diesel::associations::HasTable;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use log::{debug, error, trace};
use uuid::Uuid;

#[async_trait]
impl UserRepository for DbRepository {
    async fn add_user(&self, user: User) -> Result<(), RepoError> {
        trace!("Adding user: {:?}", user);
        let conn = &mut self.get_conn()?;
        let new_user = User {
            id: user.id,
            username: user.username,
            email: user.email,
        };
        diesel::insert_into(users::table())
            .values(&new_user)
            .execute(conn)
            .map_err(|e| {
                error!("Failed to add user: {}", e);
                RepoError::DbError(DbError::QueryError(e.to_string()))
            })?;
        debug!("User added successfully: {:?}", new_user);
        Ok(())
    }

    async fn get_all_users(&self) -> Result<Vec<User>, RepoError> {
        debug!("Fetching all users");
        let conn = &mut self.get_conn()?;
        let result = users.load::<User>(conn).map_err(|e| {
            error!("Failed to fetch all users: {}", e);
            RepoError::DbError(DbError::QueryError(e.to_string()))
        })?;
        trace!("Fetched all users successfully: {:?}", result);
        Ok(result)
    }

    async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>, RepoError> {
        debug!("Fetching user with ID: {}", user_id);
        let conn = &mut self.get_conn()?;
        let result = users
            .filter(id.eq(user_id))
            .first::<User>(conn)
            .optional()
            .map_err(|e| {
                error!("Failed to fetch user: {}", e);
                RepoError::DbError(DbError::QueryError(e.to_string()))
            })?;
        debug!("Fetched user with ID {}: {:?}", user_id, result);
        Ok(result)
    }

    async fn get_user_id(&self, user_id: &Uuid) -> Result<Option<Uuid>, RepoError> {
        debug!("Fetching user ID with ID: {}", user_id);
        self.get_user(user_id).await.map(|user| {
            let user_id = user.map(|u| u.id);
            debug!("Fetched user ID with ID {:?}: {:?}", user_id, user_id);
            user_id
        })
    }

    async fn get_user_id_by_nickname(&self, nickname: &str) -> Result<Option<Uuid>, RepoError> {
        debug!("Fetching user ID with nickname: {}", nickname);
        let conn = &mut self.get_conn()?;
        let result = users
            .filter(username.eq(nickname))
            .select(id)
            .first::<Uuid>(conn)
            .optional()
            .map_err(|e| {
                error!("Failed to fetch user ID by nickname: {}", e);
                RepoError::DbError(DbError::QueryError(e.to_string()))
            })?;
        debug!("Fetched user ID by nickname {}: {:?}", nickname, result);
        Ok(result)
    }
    ///Переделать
    async fn update_user_by_id(
        &self, user_id: &Uuid, updated_user: User,
    ) -> Result<Option<()>, RepoError> {
        debug!("Updating user with ID {}: {:?}", user_id, updated_user);
        let conn = &mut self.get_conn()?;
        let target = users.filter(id.eq(user_id));
        let updated_rows = diesel::update(target)
            .set((
                username.eq(updated_user.username),
                email.eq(updated_user.email),
            ))
            .execute(conn)
            .map_err(|e| {
                error!("Failed to update user with ID {}: {}", user_id, e);
                RepoError::DbError(DbError::QueryError(e.to_string()))
            })?;

        if updated_rows > 0 {
            debug!("User with ID {} updated successfully", user_id);
            Ok(Some(()))
        } else {
            debug!("No rows updated for user with ID {}", user_id);
            Ok(None)
        }
    }

    ///Переделать
    async fn update_user_by_nickname(
        &self, nick_name: &str, updated_user: User,
    ) -> Result<Option<()>, RepoError> {
        debug!(
            "Updating user with nickname {}: {:?}",
            nick_name, updated_user
        );
        let conn = &mut self.get_conn()?;
        let target = users.filter(username.eq(nick_name));
        let updated_rows = diesel::update(target)
            .set((
                username.eq(updated_user.username),
                email.eq(updated_user.email),
            ))
            .execute(conn)
            .map_err(|e| {
                debug!("Failed to update user with nickname {}: {}", nick_name, e);
                RepoError::DbError(DbError::QueryError(e.to_string()))
            })?;

        if updated_rows > 0 {
            debug!("User with nickname {} updated successfully", nick_name);
            Ok(Some(()))
        } else {
            debug!("No rows updated for user with nickname {}", nick_name);
            Ok(None)
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
    use crate::errors::DbError;

    fn setup_test_db() -> Result<r2d2::Pool<ConnectionManager<PgConnection>>, DbError> {
        dotenv().ok();

        let database_url = env::var("TEST_DATABASE_URL").map_err(|_| {
            DbError::ConnectionError("TEST_DATABASE_URL must be set".to_string())
        })?;

        let db_repo = DbRepository::new(&database_url)?;

        db_repo.manage_migration()?;

        Ok(db_repo.pool)
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
        let pool = setup_test_db().expect("Failed to setup test database");
        let repo = DbRepository { pool: pool.clone() };
        let result = repo.manage_migration();
        assert!(result.is_ok(), "Migration should run successfully");
    }

    #[tokio::test]
    #[serial]
    async fn add_user() {
        let pool = setup_test_db().expect("Failed to setup test database");
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user = User {
            id: Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap(),
            username: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        let result = repo.add_user(user.clone()).await;
        assert!(result.is_ok(), "User should be added successfully");

        let all_users = repo.get_all_users().await;
        assert!(all_users.is_ok(), "Should retrieve all users successfully");
        let all_users = all_users.unwrap();
        assert_eq!(all_users.len(), 1);
        assert_eq!(all_users[0].username, "testuser");
    }

    #[tokio::test]
    #[serial]
    async fn get_user_data_by_id() {
        let pool = setup_test_db().expect("Failed to setup test database");
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            username: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        let result = repo.add_user(user.clone()).await;
        assert!(result.is_ok(), "User should be added successfully");

        let fetched_user = repo.get_user(&user_id).await;
        assert!(fetched_user.is_ok(), "Should retrieve user successfully");
        let fetched_user = fetched_user.unwrap();
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().username, "testuser");
    }

    #[tokio::test]
    #[serial]
    async fn get_user_id() {
        let pool = setup_test_db().expect("Failed to setup test database");
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            username: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        let result = repo.add_user(user.clone()).await;
        assert!(result.is_ok(), "User should be added successfully");

        let fetched_user_id = repo.get_user_id(&user_id).await;
        assert!(
            fetched_user_id.is_ok(),
            "Should retrieve user ID successfully"
        );
        let fetched_user_id = fetched_user_id.unwrap();
        assert!(fetched_user_id.is_some());
        assert_eq!(fetched_user_id.unwrap(), user_id);
    }

    #[tokio::test]
    #[serial]
    async fn get_user_id_by_nickname() {
        let pool = setup_test_db().expect("Failed to setup test database");
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            username: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        let result = repo.add_user(user.clone()).await;
        assert!(result.is_ok(), "User should be added successfully");

        let fetched_user_id = repo.get_user_id_by_nickname("testuser").await;
        assert!(
            fetched_user_id.is_ok(),
            "Should retrieve user ID by nickname successfully"
        );
        let fetched_user_id = fetched_user_id.unwrap();
        assert!(fetched_user_id.is_some());
        assert_eq!(fetched_user_id.unwrap(), user_id);
    }

    #[tokio::test]
    #[serial]
    async fn update_user_by_id() {
        let pool = setup_test_db().expect("Failed to setup test database");
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            username: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        let result = repo.add_user(user.clone()).await;
        assert!(result.is_ok(), "User should be added successfully");

        let updated_user = User {
            id: user_id,
            username: "updateduser".to_string(),
            email: "updateduser@test.com".to_string(),
        };
        let result = repo.update_user_by_id(&user_id, updated_user.clone()).await;
        assert!(result.is_ok(), "User should be updated successfully");
        let result = result.unwrap();
        assert!(result.is_some());

        let fetched_user = repo.get_user(&user_id).await;
        assert!(fetched_user.is_ok(), "Should retrieve user successfully");
        let fetched_user = fetched_user.unwrap();
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().username, "updateduser");
    }

    #[tokio::test]
    #[serial]
    async fn update_user_by_nickname() {
        let pool = setup_test_db().expect("Failed to setup test database");
        let repo = DbRepository { pool: pool.clone() };
        clear_test_db(&pool);

        let user_id = Uuid::parse_str("0189a30a-60c7-7135-b683-7d7f3783d4b7").unwrap();
        let user = User {
            id: user_id,
            username: "testuser".to_string(),
            email: "testuser@test.com".to_string(),
        };
        let result = repo.add_user(user.clone()).await;
        assert!(result.is_ok(), "User should be added successfully");

        let updated_user = User {
            id: user_id,
            username: "updateduser".to_string(),
            email: "updateduser@test.com".to_string(),
        };
        let result = repo
            .update_user_by_nickname("testuser", updated_user.clone())
            .await;
        assert!(result.is_ok(), "User should be updated successfully");
        let result = result.unwrap();
        assert!(result.is_some());

        let fetched_user = repo.get_user(&user_id).await;
        assert!(fetched_user.is_ok(), "Should retrieve user successfully");
        let fetched_user = fetched_user.unwrap();
        assert!(fetched_user.is_some());
        assert_eq!(fetched_user.unwrap().username, "updateduser");
    }
}

