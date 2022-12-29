// @generated automatically by Diesel CLI.

diesel::table! {
    queue (id) {
        id -> Integer,
        user -> Text,
        is_selected -> Bool,
        is_processed -> Bool,
        is_abandoned -> Bool,
        updated_at -> Timestamp,
    }
}
