use crate::types::User;
use tonic::async_trait;
use uuid::Uuid;

mod database;
pub mod internal;

use crate::errors::RepoError;

/// Типаж, чтобы можно было и DbRepository и InternalRepository использовать (Интерфейс)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn add_user(&self, user: User) -> Result<(), RepoError>;
    async fn get_all_users(&self) -> Result<Vec<User>, RepoError>;
    async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>, RepoError>;
    async fn get_user_id(&self, user_id: &Uuid) -> Result<Option<Uuid>, RepoError>;
    async fn get_user_id_by_nickname(&self, user_name: &str) -> Result<Option<Uuid>, RepoError>;
    async fn update_user_by_id(&self, user_id: &Uuid, updated_user: User, ) -> Result<Option<()>, RepoError>;
    async fn update_user_by_nickname(&self, nick_name: &str, updated_user: User) -> Result<Option<()>, RepoError>;
}
