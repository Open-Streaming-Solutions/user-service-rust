use crate::repo::{UserRepository, RepoError};
use crate::types::User;
use async_trait::async_trait;
use dashmap::DashMap;
use std::sync::Arc;
use uuid::Uuid;

pub struct InternalRepository {
    storage: Arc<DashMap<Uuid, User>>,
}

impl InternalRepository {
    pub fn new() -> Self {
        InternalRepository {
            storage: Arc::new(DashMap::new()),
        }
    }
}

#[async_trait]
impl UserRepository for InternalRepository {
    async fn add_user(&self, user: User) -> Result<(), RepoError> {
        self.storage.insert(user.id, user);
        Ok(())
    }

    async fn get_all_users(&self) -> Result<Vec<User>, RepoError> {
        let users: Vec<User> = self.storage.iter().map(|kv| kv.value().clone()).collect();
        Ok(users)
    }

    async fn get_user(&self, user_id: &Uuid) -> Result<Option<User>, RepoError> {
        Ok(self.storage.get(user_id).map(|user| user.clone()))
    }

    async fn get_user_id(&self, user_id: &Uuid) -> Result<Option<Uuid>, RepoError> {
        Ok(self.storage.get(user_id).map(|kv| *kv.key()))
    }

    async fn get_user_id_by_nickname(&self, user_name: &str) -> Result<Option<Uuid>, RepoError> {
        Ok(self.storage
            .iter()
            .find(|kv| kv.value().name == user_name)
            .map(|kv| *kv.key()))
    }

    async fn update_user_by_id(&self, user_id: &Uuid, updated_user: User) -> Result<Option<()>, RepoError> {
        if self.storage.contains_key(user_id) {
            self.storage.insert(*user_id, updated_user);
            Ok(Some(()))
        } else {
            Ok(None)
        }
    }

    async fn update_user_by_nickname(&self, nick_name: &str, updated_user: User) -> Result<Option<()>, RepoError> {
        if let Some(user_id) = self.get_user_id_by_nickname(nick_name).await? {
            self.update_user_by_id(&user_id, updated_user).await
        } else {
            Ok(None)
        }
    }
}
