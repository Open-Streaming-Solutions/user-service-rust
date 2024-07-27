// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Uuid,
        user_name -> Varchar,
        user_email -> Varchar,
    }
}
