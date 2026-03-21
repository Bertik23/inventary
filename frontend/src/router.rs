use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Login,
    #[at("/main")]
    MainMenu,
    #[at("/inventory")]
    Inventory,
    #[at("/add")]
    Add,
    #[at("/remove")]
    Remove,
    #[at("/selection")]
    Selection,
    #[at("/reset-password")]
    ResetPassword,
    #[at("/inventories/:id/share")]
    Share { id: String },
    #[at("/inventories/:id/custom-items")]
    CustomItems { id: String },
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Login => html! { <crate::components::login::Login /> },
        Route::MainMenu => html! { <crate::components::main_menu::MainMenu /> },
        Route::Inventory => {
            html! { <crate::components::inventory_list::InventoryList /> }
        }
        Route::Add => {
            html! { <crate::components::barcode_scanner::BarcodeScanner mode="add" /> }
        }
        Route::Remove => {
            html! { <crate::components::barcode_scanner::BarcodeScanner mode="remove" /> }
        }
        Route::Selection => {
            html! { <crate::components::inventory_selection::InventorySelection /> }
        }
        Route::ResetPassword => {
            html! { <crate::components::reset_password::ResetPassword /> }
        }
        Route::Share { id } => {
            html! { <crate::components::share_inventory::ShareInventory inventory_id={id} /> }
        }
        Route::CustomItems { id } => {
            html! { <crate::components::custom_item_manager::CustomItemManager inventory_id={id} /> }
        }
    }
}
