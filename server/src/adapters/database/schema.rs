use diesel::table;

table! {
    users (id) {
        id -> Uuid,
        user_name -> Varchar,
        user_email -> Varchar,
    }
}
