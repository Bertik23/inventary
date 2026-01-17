use yew::prelude::*;
use crate::app::AppPage;
use crate::api::{add_item, remove_item, search_products, search_inventory_items, get_product_by_barcode, AddItemRequest, RemoveItemRequest, ProductInfo};
use crate::barcode::BarcodeScanner as BarcodeScannerImpl;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub mode: String,
    pub navigate: Callback<AppPage>,
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
    
    let navigate = props.navigate.clone();
    let on_back = {
        let navigate = navigate.clone();
        Callback::from(move |_| {
            navigate.emit(AppPage::MainMenu);
        })
    };
    
    let on_search = {
        let query = search_query.clone();
        let results = search_results.clone();
        let loading = loading.clone();
        let mode = props.mode.clone();
        
        Callback::from(move |_: ()| {
            let query = query.clone();
            let results = results.clone();
            let loading = loading.clone();
            let mode = mode.clone();
            
            if query.is_empty() {
                return;
            }
            
            let query_str = (*query).clone();
            
            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let result = if mode == "remove" {
                    search_inventory_items(&query_str).await
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
        let navigate = navigate.clone();
        let mode = props.mode.clone();
        
        Callback::from({
            let selected = selected.clone();
            let quantity = quantity.clone();
            let loading = loading.clone();
            let message = message.clone();
            let navigate = navigate.clone();
            let mode = mode.clone();
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
                let navigate = navigate.clone();
                let message = message.clone();
                let loading = loading.clone();
                let qty = *quantity;
                
                wasm_bindgen_futures::spawn_local(async move {
                    let result = if mode == "add" {
                        add_item(AddItemRequest {
                            barcode: product.barcode.clone(),
                            name: Some(product.name.clone()),
                            quantity: Some(qty),
                        }).await
                    } else {
                        remove_item(RemoveItemRequest {
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
                            let navigate = navigate.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                gloo_timers::future::TimeoutFuture::new(1500).await;
                                navigate.emit(AppPage::MainMenu);
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
    
    html! {
        <div class="barcode-scanner">
            <div class="header">
                <h1>{if props.mode == "add" { "Add Item" } else { "Remove Item" }}</h1>
                <button class="btn btn-secondary" onclick={on_back}>{"Back"}</button>
            </div>
            
            if let Some(ref msg) = *message {
                <div class="message">{msg}</div>
            }
            
            <div class="barcode-input-section">
                <h2>{"Enter Barcode"}</h2>
                <div class="barcode-input-box">
                    <input
                        type="text"
                        placeholder="Enter barcode (EAN-13, UPC, etc.)"
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
                    <button class="btn btn-primary" onclick={on_barcode_input_submit.clone()} disabled={*loading || barcode_input.trim().is_empty()}>
                        {"Lookup"}
                    </button>
                </div>
            </div>
            
            <div class="scanner-section">
                <h2>{"Or Scan Barcode"}</h2>
                <button class="btn btn-primary" onclick={on_start_scan} disabled={*scanning || *loading}>
                    {"Scan Barcode"}
                </button>
                
                if *scanning {
                    <BarcodeScannerImpl on_scan={on_barcode_scanned.clone()} />
                }
            </div>
            
            <div class="search-section">
                <h2>{"Or Search by Name"}</h2>
                <div class="search-box">
                    <input
                        type="text"
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
                    <button class="btn btn-primary" onclick={Callback::from(move |_| on_search.emit(()))} disabled={*loading}>
                        {"Search"}
                    </button>
                </div>
                
                if *loading {
                    <div class="loading">{"Searching..."}</div>
                } else if !search_results.is_empty() {
                    <div class="search-results">
                        {for search_results.iter().map(|product| {
                            let product_clone = product.clone();
                            let on_select = on_product_select.clone();
                            let product_name = product.name.clone();
                            let product_brand = product.brand.clone();
                            let product_barcode = product.barcode.clone();
                            let product_image = product.image_url.clone();
                            html! {
                                <div
                                    class={if selected_product.as_ref().map(|p| p.barcode == product.barcode).unwrap_or(false) {
                                        "search-result selected"
                                    } else {
                                        "search-result"
                                    }}
                                    onclick={Callback::from(move |_| on_select.emit(product_clone.clone()))}
                                >
                                    {if let Some(ref img) = product_image {
                                        html! { <img src={img.clone()} alt="Product" /> }
                                    } else {
                                        html! {}
                                    }}
                                    <div class="product-info">
                                        <h4>{product_name}</h4>
                                        {if let Some(ref brand) = product_brand {
                                            html! { <p>{"Brand: "}{brand}</p> }
                                        } else {
                                            html! {}
                                        }}
                                        {if let Some(ref barcode) = product_barcode {
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
            
            if selected_product.is_some() {
                <div class="selected-product">
                    <h3>{"Selected Product"}</h3>
                    {if let Some(ref product) = *selected_product {
                        html! {
                            <>
                                <p><strong>{&product.name}</strong></p>
                                <label>
                                    {"Quantity: "}
                                    <input
                                        type="number"
                                        min="1"
                                        value={quantity.to_string()}
                                        oninput={Callback::from(move |e: InputEvent| {
                                            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                            if let Ok(qty) = input.value().parse::<i32>() {
                                                quantity.set(qty.max(1));
                                            }
                                        })}
                                    />
                                </label>
                                <button class="btn btn-success" onclick={on_submit} disabled={*loading}>
                                    {if props.mode == "add" { "Add to Inventory" } else { "Remove from Inventory" }}
                                </button>
                            </>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            }
        </div>
    }
}
