// @generated automatically by Diesel CLI.

diesel::table! {
    queue (user) {
        user -> Text,
        is_selected -> Bool,
        is_processed -> Bool,
        updated_at -> Timestamp,
    }
}
