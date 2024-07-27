use crate::adapters::database::schema::users;
use diesel::{Insertable, Queryable};
use uuid::Uuid;

#[derive(Debug, Clone, Queryable, Insertable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
    pub user_email: String,
}
