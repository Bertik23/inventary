// @generated automatically by Diesel CLI.

diesel::table! {
    inventories (id) {
        id -> Text,
        name -> Text,
        owner_id -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    inventory_items (id) {
        id -> Text,
        inventory_id -> Text,
        barcode -> Nullable<Text>,
        name -> Text,
        quantity -> Integer,
        product_data -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    inventory_users (inventory_id, user_id) {
        inventory_id -> Text,
        user_id -> Text,
        role -> Text,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
    }
}

diesel::joinable!(inventories -> users (owner_id));
diesel::joinable!(inventory_items -> inventories (inventory_id));
diesel::joinable!(inventory_users -> inventories (inventory_id));
diesel::joinable!(inventory_users -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    inventories,
    inventory_items,
    inventory_users,
    users,
);
