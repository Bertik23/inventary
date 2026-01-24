use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use crate::api::{fetch_inventory, InventoryItem};
use crate::app::InventoryContext;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(InventoryList)]
pub fn inventory_list(_props: &Props) -> Html {
    let items = use_state(|| Vec::<InventoryItem>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    
    let inventory_context = use_context::<InventoryContext>().expect("InventoryContext not found");
    let inventory_id = match &*inventory_context.inventory_id {
        Some(id) => id.clone(),
        None => {
            return html! { <div>{"No inventory selected"}</div> };
        }
    };
    
    {
        let items = items.clone();
        let loading = loading.clone();
        let error = error.clone();
        
        use_effect_with((), move |_| {
            let items = items.clone();
            let loading = loading.clone();
            let error = error.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                match fetch_inventory(&inventory_id).await {
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
    
    let navigator = use_navigator().unwrap();
    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::MainMenu);
        })
    };
    
    html! {
        <div class="max-w-lg mx-auto p-4 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{"Inventory"}</h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{"Back"}</button>
            </div>
            
            if *loading {
                <div class="flex justify-center p-8">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                </div>
            } else if let Some(ref err) = *error {
                <div class="p-4 bg-red-100 text-red-800 rounded-xl border border-red-200">{err}</div>
            } else {
                <div class="space-y-3">
                    {for items.iter().map(|item| {
                        html! {
                            <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-100 flex flex-col gap-1 hover:shadow-md transition">
                                <div class="flex justify-between items-start">
                                    <h3 class="font-semibold text-gray-900">{&item.name}</h3>
                                    <span class="px-2 py-1 bg-blue-100 text-blue-800 text-xs font-bold rounded-full">{"Qty: "}{item.quantity}</span>
                                </div>
                                <div class="text-sm text-gray-500">
                                    {if let Some(ref barcode) = item.barcode {
                                        html! { <span class="font-mono text-xs text-gray-400">{barcode}</span> }
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
