// @generated automatically by Diesel CLI.

diesel::table! {
    inventory_items (id) {
        id -> Text,
        barcode -> Nullable<Text>,
        name -> Text,
        quantity -> Integer,
        product_data -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}
