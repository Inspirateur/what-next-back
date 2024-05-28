// @generated automatically by Diesel CLI.

diesel::table! {
    oeuvres (id) {
        id -> Integer,
        medium -> Integer,
        title -> Text,
        synopsis -> Nullable<Text>,
        picture -> Nullable<Text>,
    }
}
