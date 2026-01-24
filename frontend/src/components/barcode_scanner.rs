use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use crate::api::{add_item, remove_item, search_products, search_inventory_items, get_product_by_barcode, AddItemRequest, RemoveItemRequest, ProductInfo};
use crate::barcode::BarcodeScanner as BarcodeScannerImpl;
use crate::app::InventoryContext;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub mode: String,
}

#[function_component(BarcodeScanner)]
pub fn barcode_scanner(props: &Props) -> Html {
    let barcode_input = use_state(|| String::new());
    let search_query = use_state(|| String::new());
    let search_results = use_state(|| Vec::<ProductInfo>::new());
    let scanning = use_state(|| false);
    let selected_product = use_state(|| Option::<ProductInfo>::None);
    let quantity = use_state(|| 1);
    let loading = use_state(|| false);
    let message = use_state(|| Option::<String>::None);
    
    let inventory_context = use_context::<InventoryContext>().expect("InventoryContext not found");
    let navigator = use_navigator().unwrap();

    let inventory_id = match &*inventory_context.inventory_id {
        Some(id) => id.clone(),
        None => {
            return html! { <div>{"No inventory selected"}</div> };
        }
    };
    
    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::MainMenu);
        })
    };
    
    let on_search = {
        let query = search_query.clone();
        let results = search_results.clone();
        let loading = loading.clone();
        let mode = props.mode.clone();
        let inventory_id = inventory_id.clone();
        
        Callback::from(move |_: ()| {
            let query = query.clone();
            let results = results.clone();
            let loading = loading.clone();
            let mode = mode.clone();
            
            if query.is_empty() {
                return;
            }
            
            let query_str = (*query).clone();
            let inventory_id = inventory_id.clone();
            
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let result = if mode == "remove" {
                    search_inventory_items(&query_str, &inventory_id).await
                } else {
                    search_products(&query_str).await
                };
                
                match result {
                    Ok(products) => {
                        results.set(products);
                        loading.set(false);
                    }
                    Err(e) => {
                        results.set(vec![]);
                        loading.set(false);
                        log::error!("Search error: {}", e);
                    }
                }
            });
        })
    };
    
    let on_start_scan = {
        let scanning = scanning.clone();
        Callback::from(move |_| {
            scanning.set(true);
            // Barcode scanning will be handled by the BarcodeScanner component
        })
    };
    
    let on_product_select = {
        let selected = selected_product.clone();
        Callback::from(move |product: ProductInfo| {
            selected.set(Some(product));
        })
    };
    
    let on_submit = {
        let selected = selected_product.clone();
        let quantity = quantity.clone();
        let loading = loading.clone();
        let message = message.clone();
        let navigator = navigator.clone();
        let mode = props.mode.clone();
        let inventory_id = inventory_id.clone();
        
        Callback::from({
            let selected = selected.clone();
            let quantity = quantity.clone();
            let loading = loading.clone();
            let message = message.clone();
            let navigator = navigator.clone();
            let mode = mode.clone();
            let inventory_id = inventory_id.clone();
            move |_| {
                let product = match selected.as_ref() {
                    Some(p) => p.clone(),
                    None => {
                        message.set(Some("Please select a product".to_string()));
                        return;
                    }
                };
                
                loading.set(true);
                let mode = mode.clone();
                let navigator = navigator.clone();
                let message = message.clone();
                let loading = loading.clone();
                let qty = *quantity;
                let inventory_id = inventory_id.clone();
                
                wasm_bindgen_futures::spawn_local(async move {
                    let result = if mode == "add" {
                        add_item(AddItemRequest {
                            inventory_id: inventory_id,
                            barcode: product.barcode.clone(),
                            name: Some(product.name.clone()),
                            quantity: Some(qty),
                        }).await
                    } else {
                        remove_item(RemoveItemRequest {
                            inventory_id: inventory_id,
                            barcode: product.barcode.clone(),
                            id: None,
                            quantity: Some(qty),
                        }).await
                    };
                    
                    loading.set(false);
                    match result {
                        Ok(_) => {
                            message.set(Some(if mode == "add" {
                                "Item added successfully!".to_string()
                            } else {
                                "Item removed successfully!".to_string()
                            }));
                            // Navigate back after a delay
                            let navigator = navigator.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                gloo_timers::future::TimeoutFuture::new(1500).await;
                                navigator.push(&Route::MainMenu);
                            });
                        }
                        Err(e) => {
                            message.set(Some(format!("Error: {}", e)));
                        }
                    }
                });
            }
        })
    };
    
    // Function to lookup product by barcode
    let lookup_by_barcode = {
        let selected = selected_product.clone();
        let loading = loading.clone();
        let message = message.clone();
        let barcode_input = barcode_input.clone();
        
        Callback::from(move |barcode: String| {
            if barcode.trim().is_empty() {
                message.set(Some("Please enter a barcode".to_string()));
                return;
            }
            
            loading.set(true);
            message.set(None);
            let selected = selected.clone();
            let loading = loading.clone();
            let message = message.clone();
            let barcode_input = barcode_input.clone();
            let barcode_trimmed = barcode.trim().to_string();
            
            wasm_bindgen_futures::spawn_local(async move {
                match get_product_by_barcode(&barcode_trimmed).await {
                    Ok(product) => {
                        selected.set(Some(product));
                        loading.set(false);
                        barcode_input.set(String::new()); // Clear input on success
                    }
                    Err(e) => {
                        message.set(Some(format!("Product not found: {}", e)));
                        loading.set(false);
                    }
                }
            });
        })
    };
    
    let on_barcode_input_submit = {
        let barcode_input = barcode_input.clone();
        let lookup = lookup_by_barcode.clone();
        
        Callback::from(move |_| {
            let barcode = (*barcode_input).clone();
            lookup.emit(barcode);
        })
    };
    
    let on_barcode_input_keypress = {
        let barcode_input = barcode_input.clone();
        let lookup = lookup_by_barcode.clone();
        
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                let barcode = (*barcode_input).clone();
                lookup.emit(barcode);
            }
        })
    };
    
    let on_barcode_scanned = {
        let scanning = scanning.clone();
        let lookup = lookup_by_barcode.clone();
        
        Callback::from(move |barcode: String| {
            scanning.set(false);
            lookup.emit(barcode);
        })
    };
    
    let on_close_selection = {
        let selected = selected_product.clone();
        Callback::from(move |_| {
            selected.set(None);
        })
    };
    
    html! {
        <div class="max-w-lg mx-auto p-4 pb-32 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{if props.mode == "add" { "Add Item" } else { "Remove Item" }}</h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{"Back"}</button>
            </div>
            
            if let Some(ref msg) = *message {
                <div class="mb-4 p-4 bg-blue-100 text-blue-800 rounded-xl shadow-sm border border-blue-200">{msg}</div>
            }
            
            <div class="mb-6 bg-white p-5 rounded-xl shadow-sm border border-gray-100">
                <h2 class="text-lg font-semibold mb-3 text-gray-700">{"Enter Barcode"}</h2>
                <div class="flex gap-2">
                    <input
                        type="text"
                        class="flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition bg-gray-50"
                        placeholder="EAN-13, UPC, etc."
                        value={(*barcode_input).clone()}
                        oninput={Callback::from({
                            let barcode_input = barcode_input.clone();
                            move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                barcode_input.set(input.value());
                            }
                        })}
                        onkeypress={on_barcode_input_keypress.clone()}
                        disabled={*loading}
                    />
                    <button class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition font-medium shadow-sm" onclick={on_barcode_input_submit.clone()} disabled={*loading || barcode_input.trim().is_empty()}>
                        {"Lookup"}
                    </button>
                </div>
            </div>
            
            <div class="mb-6 bg-white p-5 rounded-xl shadow-sm border border-gray-100">
                <h2 class="text-lg font-semibold mb-3 text-gray-700">{"Or Scan Barcode"}</h2>
                
                if !*scanning {
                    <button class="w-full py-4 bg-indigo-600 text-white rounded-xl hover:bg-indigo-700 disabled:opacity-50 transition flex items-center justify-center gap-2 font-medium shadow-sm" onclick={on_start_scan} disabled={*loading}>
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v1m6 11h2m-6 0h-2v4m0-11v3m0 0h.01M12 12h4.01M16 20h4M4 12h4m12 0h.01M5 8h2a1 1 0 001-1V5a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1zm12 0h2a1 1 0 001-1V5a1 1 0 00-1-1h-2a1 1 0 00-1 1v2a1 1 0 001 1zM5 20h2a1 1 0 001-1v-2a1 1 0 00-1-1H5a1 1 0 00-1 1v2a1 1 0 001 1z" />
                        </svg>
                        {"Start Camera"}
                    </button>
                }
                
                if *scanning {
                    <div class="relative rounded-xl overflow-hidden bg-black aspect-[4/3] shadow-inner ring-4 ring-indigo-50">
                        <BarcodeScannerImpl on_scan={on_barcode_scanned.clone()} />
                        <button class="absolute top-3 right-3 bg-white/20 hover:bg-white/40 backdrop-blur-md p-2 rounded-full text-white transition" onclick={Callback::from(move |_| scanning.set(false))}>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                            </svg>
                        </button>
                    </div>
                }
            </div>
            
            <div class="mb-6">
                <h2 class="text-lg font-semibold mb-3 text-gray-700">{"Or Search by Name"}</h2>
                <div class="flex gap-2 mb-4">
                    <input
                        type="text"
                        class="flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition bg-white shadow-sm"
                        placeholder={if props.mode == "remove" { "Search inventory..." } else { "Search for product..." }}
                        value={(*search_query).clone()}
                        oninput={Callback::from({
                            let search_query = search_query.clone();
                            move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                search_query.set(input.value());
                            }
                        })}
                        onkeypress={Callback::from({
                            let on_search = on_search.clone();
                            move |e: KeyboardEvent| {
                                if e.key() == "Enter" {
                                    e.prevent_default();
                                    on_search.emit(());
                                }
                            }
                        })}
                    />
                    <button class="px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition font-medium shadow-sm" onclick={Callback::from(move |_| on_search.emit(()))} disabled={*loading}>
                        {"Search"}
                    </button>
                </div>
                
                if *loading {
                    <div class="flex justify-center p-8">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                } else if !search_results.is_empty() {
                    <div class="space-y-3">
                        {for search_results.iter().map(|product| {
                            let product_clone = product.clone();
                            let on_select = on_product_select.clone();
                            let product_name = product.name.clone();
                            let product_brand = product.brand.clone();
                            let product_barcode = product.barcode.clone();
                            let product_image = product.image_url.clone();
                            html! {
                                <div
                                    class={classes!(
                                        "flex", "items-center", "gap-4", "p-3", "rounded-xl", "border", "cursor-pointer", "transition", "hover:shadow-md", "bg-white",
                                        if selected_product.as_ref().map(|p| p.barcode == product.barcode).unwrap_or(false) {
                                            "border-blue-500 ring-2 ring-blue-100"
                                        } else {
                                            "border-gray-200 hover:border-blue-300"
                                        }
                                    )}
                                    onclick={Callback::from(move |_| on_select.emit(product_clone.clone()))}
                                >
                                    <div class="w-16 h-16 flex-shrink-0 bg-gray-50 rounded-lg overflow-hidden flex items-center justify-center border border-gray-100">
                                        {if let Some(ref img) = product_image {
                                            html! { <img src={img.clone()} alt="Product" class="w-full h-full object-contain" /> }
                                        } else {
                                            html! { <span class="text-gray-300 text-xs">{"No Img"}</span> }
                                        }}
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <h4 class="font-medium text-gray-900 truncate">{product_name}</h4>
                                        {if let Some(ref brand) = product_brand {
                                            html! { <p class="text-sm text-gray-500 truncate">{brand}</p> }
                                        } else {
                                            html! {}
                                        }}
                                        {if let Some(ref barcode) = product_barcode {
                                            html! { <p class="text-xs text-gray-400 font-mono mt-1">{barcode}</p> }
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
            
            if selected_product.is_some() {
                <div class="fixed bottom-0 left-0 right-0 bg-white border-t border-gray-200 p-4 shadow-[0_-4px_6px_-1px_rgba(0,0,0,0.1)] z-50">
                    <div class="max-w-lg mx-auto">
                        <div class="flex justify-between items-start mb-4">
                            <h3 class="font-semibold text-gray-800">{"Selected Product"}</h3>
                            <button class="text-gray-400 hover:text-gray-600" onclick={on_close_selection}>
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </div>
                        
                        {if let Some(ref product) = *selected_product {
                            html! {
                                <div class="space-y-4">
                                    <p class="text-sm text-gray-600 truncate"><strong>{&product.name}</strong></p>
                                    <div class="flex items-center gap-4">
                                        <label class="flex items-center gap-2 text-gray-700 font-medium">
                                            {"Qty:"}
                                            <input
                                                type="number"
                                                min="1"
                                                class="w-20 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none text-center"
                                                value={quantity.to_string()}
                                                oninput={Callback::from(move |e: InputEvent| {
                                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                    if let Ok(qty) = input.value().parse::<i32>() {
                                                        quantity.set(qty.max(1));
                                                    }
                                                })}
                                            />
                                        </label>
                                        <button 
                                            class={classes!(
                                                "flex-1", "py-3", "px-4", "rounded-lg", "text-white", "font-medium", "transition", "shadow-sm",
                                                if props.mode == "add" { "bg-green-600 hover:bg-green-700" } else { "bg-red-600 hover:bg-red-700" }
                                            )}
                                            onclick={on_submit} 
                                            disabled={*loading}
                                        >
                                            {if props.mode == "add" { "Add to Inventory" } else { "Remove" }}
                                        </button>
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                </div>
            }
        </div>
    }
}
