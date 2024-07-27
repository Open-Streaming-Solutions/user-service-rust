
use uuid::Uuid;
use crate::app::structs::User;
use crate::adapters::UserRepository;
use async_trait::async_trait;
use diesel::{PgConnection, r2d2};
use diesel::r2d2::ConnectionManager;

/*
Читается так:
Определение алиаса Pool для библиотечного типа Pool, Который содершит структуру Для подключения к БД postgresql
*/
type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub struct DbRepository {
    pool: Pool,
}

impl DbRepository {
    pub fn new(_database_url: &str) -> Self {
       unimplemented!()
    }

    fn get_conn(&self)  {
        unimplemented!()
    }
}
/*
Для DbRepository Можно не использовать static, так как Pool з
*/
#[async_trait]
impl UserRepository for DbRepository {
    async fn add_user(&self, _user: User) {
        unimplemented!()
    }

    async fn get_user(&self, _user_id: &Uuid) -> Option<User> {
        unimplemented!()
    }

    async fn get_user_id(&self, _user_id: &Uuid) -> Option<Uuid> {
        unimplemented!()
    }

    async fn get_user_id_by_nickname(&self, _user_name: &str) -> Option<Uuid> {
        unimplemented!()
    }

    async fn get_all_users(&self) -> Vec<User> {
        unimplemented!()
    }
}
