use diesel::Queryable;
use uuid::Uuid;

#[derive(Debug, Clone,Queryable)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
    pub user_email: String,
}
