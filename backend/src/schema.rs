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
    custom_products (barcode) {
        barcode -> Text,
        name -> Text,
        brand -> Nullable<Text>,
        image_url -> Nullable<Text>,
        unit -> Nullable<Text>,
        created_at -> Timestamp,
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
    inventory_categories (id) {
        id -> Text,
        inventory_id -> Text,
        name -> Text,
        parent_id -> Nullable<Text>,
    }
}

diesel::table! {
    inventory_items (id) {
        id -> Text,
        inventory_id -> Text,
        barcode -> Nullable<Text>,
        name -> Text,
        quantity -> Float,
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
    item_categories (item_id, category_id) {
        item_id -> Text,
        category_id -> Text,
    }
}

diesel::table! {
    pending_products (barcode) {
        barcode -> Text,
        name -> Text,
        brand -> Nullable<Text>,
        unit -> Nullable<Text>,
        added_by -> Text,
        status -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Text,
        username -> Text,
        password_hash -> Text,
        created_at -> Timestamp,
        email -> Nullable<Text>,
        reset_token -> Nullable<Text>,
        reset_token_expiry -> Nullable<Timestamp>,
        role -> Text,
    }
}

diesel::joinable!(inventories -> users (owner_id));
diesel::joinable!(inventory_categories -> inventories (inventory_id));
diesel::joinable!(inventory_items -> inventories (inventory_id));
diesel::joinable!(inventory_users -> inventories (inventory_id));
diesel::joinable!(inventory_users -> users (user_id));
diesel::joinable!(item_categories -> inventory_categories (category_id));
diesel::joinable!(item_categories -> inventory_items (item_id));
diesel::joinable!(pending_products -> users (added_by));

diesel::allow_tables_to_appear_in_same_query!(
    custom_item_templates,
    custom_products,
    inventories,
    inventory_categories,
    inventory_items,
    inventory_users,
    item_categories,
    pending_products,
    users,
);
