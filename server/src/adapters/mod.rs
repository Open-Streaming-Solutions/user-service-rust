use crate::app::structs::User;
use tonic::async_trait;
use uuid::Uuid;

pub mod database;
pub mod repo;

// Типаж, чтобы можно было и DbRepository и InternalRepository использовать (Интерфейс)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn add_user(&self, user: User);
    async fn get_all_users(&self) -> Vec<User>;
    async fn get_user(&self, user_id: &Uuid) -> Option<User>;
    async fn get_user_id(&self, user_id: &Uuid) -> Option<Uuid>;
    async fn get_user_id_by_nickname(&self, user_name: &str) -> Option<Uuid>;
    async fn update_user_by_id(&self, user_id: &Uuid, updated_user: User) -> Option<()>;
    async fn update_user_by_nickname(&self, nick_name: &str, updated_user: User) -> Option<()>;
}
