use yew::prelude::*;
use crate::app::AppPage;
use crate::api::{fetch_inventory, InventoryItem};

#[derive(Properties, PartialEq)]
pub struct Props {
    pub navigate: Callback<AppPage>,
}

#[function_component(InventoryList)]
pub fn inventory_list(props: &Props) -> Html {
    let items = use_state(|| Vec::<InventoryItem>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    
    {
        let items = items.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            let items = items.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_inventory().await {
                    Ok(inventory) => {
                        items.set(inventory);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        });
    }
    
    let navigate = props.navigate.clone();
    let on_back = Callback::from(move |_| {
        navigate.emit(AppPage::MainMenu);
    });
    
    html! {
        <div class="inventory-list">
            <div class="header">
                <h1>{"Inventory"}</h1>
                <button class="btn btn-secondary" onclick={on_back}>{"Back"}</button>
            </div>
            
            if *loading {
                <div class="loading">{"Loading..."}</div>
            } else if let Some(ref err) = *error {
                <div class="error">{err}</div>
            } else {
                <div class="items-list">
                    {for items.iter().map(|item| {
                        html! {
                            <div class="inventory-item">
                                <div class="item-info">
                                    <h3>{&item.name}</h3>
                                    <p>{"Quantity: "}{item.quantity}</p>
                                    {if let Some(ref barcode) = item.barcode {
                                        html! { <p>{"Barcode: "}{barcode}</p> }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        </div>
    }
}
