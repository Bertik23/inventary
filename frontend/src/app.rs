use yew::prelude::*;
use crate::components::{MainMenu, InventoryList, BarcodeScanner};

#[derive(Clone, PartialEq)]
pub enum AppPage {
    MainMenu,
    InventoryList,
    AddItem,
    RemoveItem,
}

#[function_component(App)]
pub fn app() -> Html {
    let page = use_state(|| AppPage::MainMenu);
    
    let navigate = {
        let page = page.clone();
        Callback::from(move |new_page: AppPage| {
            page.set(new_page);
        })
    };
    
    html! {
        <div class="app">
            {match *page {
                AppPage::MainMenu => html! {
                    <MainMenu {navigate} />
                },
                AppPage::InventoryList => html! {
                    <InventoryList {navigate} />
                },
                AppPage::AddItem => html! {
                    <BarcodeScanner mode="add" {navigate} />
                },
                AppPage::RemoveItem => html! {
                    <BarcodeScanner mode="remove" {navigate} />
                },
            }}
        </div>
    }
}
