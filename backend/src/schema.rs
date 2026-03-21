// @generated automatically by Diesel CLI.

diesel::table! {
    custom_item_templates (id) {
        id -> Text,
        inventory_id -> Nullable<Text>,
        name -> Text,
        default_unit -> Text,
    }
}

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
        quantity -> Double,
        unit -> Text,
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
        email -> Text,
        password_hash -> Text,
        reset_token -> Nullable<Text>,
        reset_token_expiry -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::joinable!(inventories -> users (owner_id));
diesel::joinable!(inventory_items -> inventories (inventory_id));
diesel::joinable!(inventory_users -> inventories (inventory_id));
diesel::joinable!(inventory_users -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    custom_item_templates,
    inventories,
    inventory_items,
    inventory_users,
    users,
);
