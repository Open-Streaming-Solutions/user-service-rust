use diesel::table;

table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        email -> Varchar,
    }
}
