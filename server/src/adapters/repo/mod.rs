use dashmap::DashMap;
use std::sync::Arc;
use crate::app::structs::User;
use uuid::Uuid;

//Возможно будет полезен далее
pub struct InternalRepository {
    storage: Arc<DashMap<Uuid, User>>,
}

impl InternalRepository {
    pub fn new() -> Self {
        InternalRepository {
            storage: Arc::new(DashMap::new()),
        }
    }

    pub fn add_user(&self, user: User) {
        self.storage.insert(user.id, user);
    }

    pub fn get_user(&self, user_id: &Uuid) -> Option<User> {
        self.storage.get(user_id).map(|user| user.clone())
    }

    pub fn get_user_id(&self, user_id: &Uuid) -> Option<Uuid> {
        self.storage.get(user_id).map(|kv| *kv.key())
    }
    pub fn get_user_id_by_nickname(&self, user_name: &str) -> Option<Uuid> {
        self.storage.iter()
            .find(|kv| kv.value().user_name == user_name)
            .map(|kv| *kv.key())
    }

    pub fn get_all_users(&self) -> Vec<User> {
        self.storage.iter().map(|kv| kv.value().clone()).collect()
    }
}