use yew::prelude::*;
use crate::app::AppPage;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub navigate: Callback<AppPage>,
}

#[function_component(MainMenu)]
pub fn main_menu(props: &Props) -> Html {
    let navigate = props.navigate.clone();
    
    let on_add_click = {
        let navigate = navigate.clone();
        Callback::from(move |_| {
            navigate.emit(AppPage::AddItem);
        })
    };
    
    let on_remove_click = {
        let navigate = navigate.clone();
        Callback::from(move |_| {
            navigate.emit(AppPage::RemoveItem);
        })
    };
    
    let on_show_inventory_click = {
        let navigate = navigate.clone();
        Callback::from(move |_| {
            navigate.emit(AppPage::InventoryList);
        })
    };
    
    html! {
        <div class="main-menu">
            <h1>{"Inventory Management"}</h1>
            <div class="button-group">
                <button class="btn btn-primary" onclick={on_add_click}>
                    {"Add Item"}
                </button>
                <button class="btn btn-danger" onclick={on_remove_click}>
                    {"Remove Item"}
                </button>
                <button class="btn btn-secondary" onclick={on_show_inventory_click}>
                    {"Show Inventory"}
                </button>
            </div>
        </div>
    }
}
