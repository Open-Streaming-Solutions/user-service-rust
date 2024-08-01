use crate::adapters::schema::users;
use diesel::{Insertable, Queryable};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable, PartialEq)]
#[diesel(table_name = users)]
pub struct User {
    id: Uuid,
    username: String,
    email: String,
}

impl User {
    pub async fn new(id: Uuid, username: String, email: String) -> Self {
        User {
            id,
            username,
            email,
        }
    }
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }

    pub fn get_email(&self) -> &str {
        &self.email
    }
    pub fn set_username(&mut self, username: String) {
        self.username = username;
    }

    pub fn set_email(&mut self, email: String) {
        self.email = email;
    }
    pub fn set_id(&mut self, id: Uuid) {
        self.id = id;
    }
}