use tonic::async_trait;
use uuid::Uuid;
use crate::app::structs::User;

pub mod repo;
mod database;

// Типаж, чтобы можно было и DbRepository и InternalRepository использовать (Интерфейс)
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn add_user(&self, user: User);
    async fn get_user(&self, user_id: &Uuid) -> Option<User>;
    async fn get_user_id(&self, user_id: &Uuid) -> Option<Uuid>;
    async fn get_user_id_by_nickname(&self, user_name: &str) -> Option<Uuid>;
    async fn get_all_users(&self) -> Vec<User>;
}