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

diesel::joinable!(imdb_map -> oeuvres (oeuvre_id));
diesel::joinable!(oeuvre_tags -> oeuvres (oeuvre_id));
diesel::joinable!(oeuvre_tags -> tags (tag_id));

diesel::allow_tables_to_appear_in_same_query!(
    imdb_map,
    oeuvre_tags,
    oeuvres,
    tags,
);
