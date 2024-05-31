// @generated automatically by Diesel CLI.

diesel::table! {
    imdb_map (oeuvre_id, imdb_id) {
        oeuvre_id -> Integer,
        imdb_id -> Text,
    }
}

diesel::table! {
    oeuvre_tags (oeuvre_id, tag_id) {
        oeuvre_id -> Integer,
        tag_id -> Integer,
    }
}

diesel::table! {
    oeuvres (id) {
        id -> Integer,
        medium -> Integer,
        title -> Text,
        rating -> Nullable<Integer>,
        synopsis -> Nullable<Text>,
        picture -> Nullable<Text>,
    }
}

diesel::table! {
    tags (id) {
        id -> Integer,
        label -> Text,
    }
}

diesel::table! {
    user_ratings (user_id, oeuvre_id) {
        user_id -> Integer,
        oeuvre_id -> Integer,
        rating -> Integer,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        pwd_hash -> Binary,
        pwd_salt -> Binary,
    }
}

diesel::joinable!(imdb_map -> oeuvres (oeuvre_id));
diesel::joinable!(oeuvre_tags -> oeuvres (oeuvre_id));
diesel::joinable!(oeuvre_tags -> tags (tag_id));
diesel::joinable!(user_ratings -> oeuvres (oeuvre_id));
diesel::joinable!(user_ratings -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    imdb_map,
    oeuvre_tags,
    oeuvres,
    tags,
    user_ratings,
    users,
);
