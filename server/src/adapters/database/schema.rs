use diesel::table;

table! {
        users (id) {
            id -> Uuid,
            name -> Varchar,
            email -> Varchar,
        }
    }