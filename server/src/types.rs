use crate::adapters::schema::users;
use diesel::{Insertable, Queryable};
use uuid::Uuid;
use crate::adapters::schema::users::{email, id, username};

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
}