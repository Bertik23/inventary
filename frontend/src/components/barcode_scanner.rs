use crate::api::{
    add_item, buffer_unknown_product, get_inventory_categories,
    get_product_by_barcode, remove_item, search_inventory_items,
    search_products, AddItemRequest, BufferProductRequest, InventoryCategory,
    ProductInfo, RemoveItemRequest,
};
use crate::app::{InventoryContext, UserContext};
use crate::barcode::BarcodeScanner as ScannerComponent;
use crate::i18n::use_i18n;
use crate::router::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub mode: String, // "add" or "remove"
}

use crate::format_quantity;

#[function_component(BarcodeScanner)]
pub fn barcode_scanner(props: &Props) -> Html {
    let inventory_context =
        use_context::<InventoryContext>().expect("InventoryContext not found");
    let user_context =
        use_context::<UserContext>().expect("UserContext not found");
    let inventory_id = (*inventory_context.inventory_id)
        .clone()
        .unwrap_or_default();
    let navigator = use_navigator().unwrap();
    let i18n = use_i18n();

    let scanning = use_state(|| false);
    let barcode_input = use_state(|| String::new());
    let selected_product = use_state(|| Option::<ProductInfo>::None);
    let quantity = use_state(|| 1.0);
    let selected_unit = use_state(|| "pcs".to_string());
    let loading = use_state(|| false);
    let message = use_state(|| Option::<String>::None);

    let search_query = use_state(|| String::new());
    let search_results = use_state(|| Vec::<ProductInfo>::new());
    let templates = use_state(|| Vec::<crate::api::CustomItemTemplate>::new());

    // Unknown product states
    let is_unknown = use_state(|| false);
    let unknown_barcode = use_state(|| String::new());
    let unknown_name = use_state(|| String::new());
    let unknown_brand = use_state(|| String::new());
    let unknown_unit = use_state(|| "pcs".to_string());
    let available_categories = use_state(|| Vec::<InventoryCategory>::new());
    let selected_category_ids = use_state(|| Vec::<String>::new());

    {
        let templates = templates.clone();
        let inventory_id = inventory_id.clone();
        let available_categories = available_categories.clone();
        use_effect_with(inventory_id, move |inventory_id| {
            let inventory_id = inventory_id.clone();
            let inventory_id_for_cats = inventory_id.clone();
            let available_categories = available_categories.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match crate::api::get_custom_item_templates(Some(&inventory_id))
                    .await
                {
                    Ok(t) => templates.set(t),
                    Err(e) => log::error!("Failed to fetch templates: {}", e),
                }

                match get_inventory_categories(&inventory_id_for_cats).await {
                    Ok(cats) => available_categories.set(cats),
                    Err(e) => log::error!("Failed to fetch categories: {}", e),
                }
            });
        });
    }

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
                    search_products(&query_str, Some(&inventory_id)).await
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
        })
    };

    let on_product_select = {
        let selected = selected_product.clone();
        let selected_unit = selected_unit.clone();
        Callback::from(move |product: ProductInfo| {
            if let Some(ref u) = product.unit {
                selected_unit.set(u.clone());
            } else {
                selected_unit.set("pcs".to_string());
            }
            selected.set(Some(product));
        })
    };

    let on_submit = {
        let selected = selected_product.clone();
        let quantity = quantity.clone();
        let selected_unit = selected_unit.clone();
        let loading = loading.clone();
        let message = message.clone();
        let navigator = navigator.clone();
        let mode = props.mode.clone();
        let inventory_id = inventory_id.clone();
        let i18n = i18n.clone();
        let selected_category_ids = selected_category_ids.clone();

        Callback::from(move |_| {
            if let Some(ref product) = *selected {
                loading.set(true);
                let req_inventory_id = inventory_id.clone();
                let barcode = product.barcode.clone();
                let name = Some(product.name.clone());
                let qty = Some(*quantity as f32);
                let unit = Some((*selected_unit).clone());
                let cats = if (*selected_category_ids).is_empty() {
                    None
                } else {
                    Some((*selected_category_ids).clone())
                };

                let loading = loading.clone();
                let message = message.clone();
                let navigator = navigator.clone();
                let mode = mode.clone();
                let i18n = i18n.clone();
                let product_id = product.id.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let result = if mode == "add" {
                        add_item(AddItemRequest {
                            inventory_id: req_inventory_id,
                            barcode,
                            name,
                            quantity: qty,
                            unit,
                            categories: cats,
                        })
                        .await
                    } else {
                        remove_item(RemoveItemRequest {
                            inventory_id: req_inventory_id,
                            barcode,
                            id: product_id,
                            name,
                            quantity: qty,
                        })
                        .await
                    };

                    loading.set(false);
                    match result {
                        Ok(_) => {
                            let msg = if mode == "add" {
                                i18n.t("barcode.item_added")
                            } else {
                                i18n.t("barcode.item_removed")
                            };
                            message.set(Some(msg));
                            gloo_timers::future::TimeoutFuture::new(1500).await;
                            navigator.push(&Route::MainMenu);
                        }
                        Err(e) => {
                            message.set(Some(
                                i18n.t_with("common.error", vec![("e", &e)]),
                            ));
                        }
                    }
                });
            }
        })
    };

    let on_submit_unknown = {
        let unknown_barcode = unknown_barcode.clone();
        let unknown_name = unknown_name.clone();
        let unknown_brand = unknown_brand.clone();
        let unknown_unit = unknown_unit.clone();
        let loading = loading.clone();
        let message = message.clone();
        let navigator = navigator.clone();
        let inventory_id = inventory_id.clone();
        let user_id = match &*user_context.user {
            Some(u) => u.id.clone(),
            None => String::new(),
        };
        let i18n = i18n.clone();

        Callback::from(move |_| {
            let barcode = (*unknown_barcode).clone();
            let name = (*unknown_name).clone();
            let brand = (*unknown_brand).clone();
            let unit = (*unknown_unit).clone();

            if name.is_empty() {
                return;
            }

            loading.set(true);
            let inv_id = inventory_id.clone();
            let user_id = user_id.clone();
            let loading = loading.clone();
            let message = message.clone();
            let navigator = navigator.clone();
            let i18n = i18n.clone();

            wasm_bindgen_futures::spawn_local(async move {
                // 1. Buffer the product
                let buffer_req = BufferProductRequest {
                    barcode: barcode.clone(),
                    name: name.clone(),
                    brand: if brand.is_empty() {
                        None
                    } else {
                        Some(brand.clone())
                    },
                    unit: Some(unit.clone()),
                    added_by: user_id,
                };
                let _ = buffer_unknown_product(buffer_req).await;

                // 2. Add to inventory
                let add_req = AddItemRequest {
                    inventory_id: inv_id,
                    barcode: Some(barcode),
                    name: Some(name),
                    quantity: Some(1.0 as f32),
                    unit: Some(unit),
                    categories: None,
                };
                match add_item(add_req).await {
                    Ok(_) => {
                        message.set(Some(i18n.t("barcode.item_added")));
                        gloo_timers::future::TimeoutFuture::new(1500).await;
                        navigator.push(&Route::MainMenu);
                    }
                    Err(e) => {
                        message.set(Some(
                            i18n.t_with("common.error", vec![("e", &e)]),
                        ));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let lookup_by_barcode = {
        let selected = selected_product.clone();
        let loading = loading.clone();
        let message = message.clone();
        let barcode_input = barcode_input.clone();
        let selected_unit = selected_unit.clone();
        let is_unknown = is_unknown.clone();
        let unknown_barcode = unknown_barcode.clone();
        let unknown_name = unknown_name.clone();
        let inventory_id = inventory_id.clone();
        let i18n = i18n.clone();

        Callback::from(move |barcode: String| {
            if barcode.trim().is_empty() {
                message.set(Some(i18n.t("barcode.enter_barcode")));
                return;
            }

            loading.set(true);
            message.set(None);
            let selected = selected.clone();
            let loading = loading.clone();
            let barcode_input = barcode_input.clone();
            let barcode_trimmed = barcode.trim().to_string();
            let inv_id = inventory_id.clone();
            let selected_unit = selected_unit.clone();
            let is_unknown = is_unknown.clone();
            let unknown_barcode = unknown_barcode.clone();
            let unknown_name = unknown_name.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match get_product_by_barcode(&barcode_trimmed, Some(&inv_id))
                    .await
                {
                    Ok(product) => {
                        if let Some(ref u) = product.unit {
                            selected_unit.set(u.clone());
                        } else {
                            selected_unit.set("pcs".to_string());
                        }
                        selected.set(Some(product));
                        loading.set(false);
                        barcode_input.set(String::new());
                    }
                    Err(_) => {
                        // Product not found - show unknown form
                        unknown_barcode.set(barcode_trimmed);
                        unknown_name.set(String::new());
                        is_unknown.set(true);
                        loading.set(false);
                    }
                }
            });
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
        let is_unknown = is_unknown.clone();
        Callback::from(move |_| {
            selected.set(None);
            is_unknown.set(false);
        })
    };

    html! {
        <div class="max-w-lg mx-auto p-4 min-h-screen bg-gray-50 pb-32">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">
                    {if props.mode == "add" { i18n.t("main_menu.add_item") } else { i18n.t("main_menu.remove_item") }}
                </h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{i18n.t("common.back")}</button>
            </div>

            if let Some(ref msg) = *message {
                <div class="mb-4 p-4 bg-blue-100 text-blue-800 rounded-xl border border-blue-200">{msg}</div>
            }

            <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 mb-6">
                <h2 class="text-lg font-semibold mb-3 text-gray-700">{i18n.t("barcode.scan_barcode")}</h2>

                if *scanning {
                    <div class="relative rounded-xl overflow-hidden bg-black aspect-square mb-4">
                        <ScannerComponent on_scan={on_barcode_scanned} />
                        <button
                            class="absolute bottom-4 left-1/2 -translate-x-1/2 px-6 py-2 bg-red-600 text-white rounded-full font-medium shadow-lg"
                            onclick={let scanning = scanning.clone(); move |_| scanning.set(false)}
                        >
                            {i18n.t("barcode.stop_scanning")}
                        </button>
                    </div>
                } else {
                    <button
                        class="w-full py-12 border-2 border-dashed border-gray-300 rounded-xl flex flex-col items-center justify-center text-gray-500 hover:border-blue-500 hover:text-blue-600 transition group bg-gray-50"
                        onclick={on_start_scan}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-12 w-12 mb-2 group-hover:scale-110 transition-transform" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 9a2 2 0 012-2h.93a2 2 0 001.664-.89l.812-1.22A2 2 0 0110.07 4h3.86a2 2 0 011.664.89l.812 1.22A2 2 0 0018.07 7H19a2 2 0 012 2v9a2 2 0 01-2 2H5a2 2 0 01-2-2V9z" />
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 13a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                        <span class="font-medium">{i18n.t("barcode.start_camera")}</span>
                    </button>
                }
            </div>

            <div class="bg-white p-6 rounded-2xl shadow-sm border border-gray-100 mb-6">
                <h2 class="text-lg font-semibold mb-3 text-gray-700">{i18n.t("barcode.enter_barcode")}</h2>
                <div class="flex flex-col sm:flex-row gap-2">
                    <input
                        type="text"
                        class="flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition bg-gray-50"
                        placeholder={i18n.t("barcode.barcode_placeholder")}
                        value={(*barcode_input).clone()}
                        oninput={Callback::from({
                            let barcode_input = barcode_input.clone();
                            move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                barcode_input.set(input.value());
                            }
                        })}
                        onkeypress={on_barcode_input_keypress}
                    />
                    <button
                        class="w-full sm:w-auto px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium shadow-sm"
                        onclick={let barcode_input = barcode_input.clone(); let lookup = lookup_by_barcode.clone(); move |_| lookup.emit((*barcode_input).clone())}
                        disabled={*loading}
                    >
                        {i18n.t("barcode.lookup")}
                    </button>
                </div>
            </div>

            if !templates.is_empty() {
                <div class="mb-6">
                    <h2 class="text-lg font-semibold mb-3 text-gray-700">{i18n.t("barcode.quick_add")}</h2>
                    <div class="flex flex-wrap gap-2">
                        {for templates.iter().map(|template| {
                            let template_clone = template.clone();
                            let on_select = on_product_select.clone();
                            html! {
                                <button
                                    onclick={Callback::from(move |_| on_select.emit(ProductInfo {
                                        id: None,
                                        barcode: None,
                                        name: template_clone.name.clone(),
                                        brand: None,
                                        image_url: None,
                                        categories: vec![],
                                        unit: Some(template_clone.default_unit.clone()),
                                    }))}
                                    class="px-3 sm:px-4 py-2 bg-blue-50 text-blue-700 rounded-full border border-blue-100 hover:bg-blue-100 transition font-medium text-xs sm:text-sm whitespace-nowrap overflow-hidden text-ellipsis max-w-[150px] sm:max-w-none"
                                    title={template.name.clone()}
                                >
                                    {&template.name}
                                </button>
                            }
                        })}
                    </div>
                </div>
            }

            <div class="mb-6">
                <h2 class="text-lg font-semibold mb-3 text-gray-700">{i18n.t("barcode.search_by_name")}</h2>
                <div class="flex flex-col sm:flex-row gap-2 mb-4">
                    <input
                        type="text"
                        class="flex-1 px-4 py-3 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition bg-white shadow-sm"
                        placeholder={if props.mode == "remove" { i18n.t("barcode.search_inventory") } else { i18n.t("barcode.search_product") }}
                        value={(*search_query).clone()}
                        oninput={Callback::from({
                            let search_query = search_query.clone();
                            move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                search_query.set(input.value());
                            }
                        })}
                        onkeypress={let on_search = on_search.clone(); move |e: KeyboardEvent| if e.key() == "Enter" { on_search.emit(()); }}
                    />
                    <button
                        class="w-full sm:w-auto px-6 py-3 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium shadow-sm"
                        onclick={let on_search = on_search.clone(); move |_| on_search.emit(())}
                        disabled={*loading}
                    >
                        {i18n.t("barcode.search")}
                    </button>
                </div>

                if !search_results.is_empty() {
                    <div class="space-y-2 mb-6">
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
                                        "flex", "items-center", "gap-4", "p-3", "bg-white", "border", "rounded-xl", "cursor-pointer", "transition-all", "active:scale-95",
                                        if selected_product.as_ref().map(|p| p.barcode.as_ref()) == Some(product.barcode.as_ref()) {
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
                                            html! { <span class="text-gray-300 text-xs">{i18n.t("barcode.no_img")}</span> }
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
                            <h3 class="font-semibold text-gray-800">{i18n.t("barcode.selected_product")}</h3>
                            <button class="text-gray-400 hover:text-gray-600" onclick={on_close_selection.clone()}>
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd" />
                                </svg>
                            </button>
                        </div>

                        {if let Some(ref product) = *selected_product {
                            let quantity_val = *quantity;
                            let unit_val = (*selected_unit).clone();
                            html! {
                                <div class="space-y-4">
                                    <p class="text-sm text-gray-600 truncate"><strong>{&product.name}</strong></p>
                                    <div class="flex flex-wrap items-center gap-4">
                                        <div class="flex items-center gap-2">
                                            <span class="text-gray-700 font-medium">{i18n.t("common.qty_label")}</span>
                                            <input
                                                type="number"
                                                step="0.1"
                                                min="0.1"
                                                class="w-24 px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none text-center"
                                                value={format_quantity(quantity_val)}
                                                oninput={Callback::from({
                                                    let quantity = quantity.clone();
                                                    move |e: InputEvent| {
                                                        let input: HtmlInputElement = e.target_unchecked_into();
                                                        if let Ok(qty) = input.value().parse::<f64>() {
                                                            quantity.set(qty);
                                                        }
                                                    }
                                                })}
                                            />
                                        </div>
                                        <div class="flex items-center gap-2">
                                            <span class="text-gray-700 font-medium">{i18n.t("common.unit")}</span>
                                            {if product.unit.is_some() {
                                                html! { <span class="px-3 py-2 bg-gray-100 text-gray-700 rounded-lg border border-gray-200 font-medium">{unit_val.clone()}</span> }
                                            } else {
                                                html! {
                                                    <select
                                                        class="px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none"
                                                        value={unit_val.clone()}
                                                        onchange={Callback::from({
                                                            let selected_unit = selected_unit.clone();
                                                            move |e: Event| {
                                                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                                selected_unit.set(input.value());
                                                            }
                                                        })}
                                                    >
                                                        <option value="pcs" selected={unit_val == "pcs"}>{i18n.t("custom_items.unit_pcs")}</option>
                                                        <option value="kg" selected={unit_val == "kg"}>{i18n.t("custom_items.unit_kg")}</option>
                                                        <option value="g" selected={unit_val == "g"}>{i18n.t("custom_items.unit_g")}</option>
                                                        <option value="l" selected={unit_val == "l"}>{i18n.t("custom_items.unit_l")}</option>
                                                        <option value="ml" selected={unit_val == "ml"}>{i18n.t("custom_items.unit_ml")}</option>
                                                    </select>
                                                }
                                            }}
                                        </div>

                                        if !available_categories.is_empty() && props.mode == "add" {
                                            <div class="space-y-2 mt-4">
                                                <span class="text-gray-700 font-medium block">{i18n.t("Categories")}</span>
                                                <div class="flex flex-wrap gap-2 max-h-32 overflow-y-auto p-2 border border-gray-200 rounded-lg">
                                                    {for available_categories.iter().map(|cat| {
                                                        let cat_id = cat.id.clone();
                                                        let selected_ids = selected_category_ids.clone();
                                                        let is_selected = (*selected_ids).contains(&cat_id);
                                                        let on_toggle = {
                                                            let cat_id = cat_id.clone();
                                                            let selected_ids = selected_ids.clone();
                                                            Callback::from(move |_| {
                                                                let mut current = (*selected_ids).clone();
                                                                if current.contains(&cat_id) {
                                                                    current.retain(|id| id != &cat_id);
                                                                } else {
                                                                    current.push(cat_id.clone());
                                                                }
                                                                selected_ids.set(current);
                                                            })
                                                        };
                                                        html! {
                                                            <button
                                                                onclick={on_toggle}
                                                                class={classes!(
                                                                    "px-3", "py-1", "text-xs", "rounded-full", "border", "transition",
                                                                    if is_selected { "bg-blue-600 text-white border-blue-600" } else { "bg-white text-gray-600 border-gray-200 hover:border-blue-300" }
                                                                )}
                                                            >
                                                                {&cat.name}
                                                            </button>
                                                        }
                                                    })}
                                                </div>
                                            </div>
                                        }

                                        <button
                                            class={classes!(
                                                "flex-1", "py-3", "px-4", "rounded-lg", "text-white", "font-medium", "transition", "shadow-sm",
                                                if props.mode == "add" { "bg-green-600 hover:bg-green-700" } else { "bg-red-600 hover:bg-red-700" }
                                            )}
                                            onclick={on_submit}
                                            disabled={*loading}
                                        >
                                            {if props.mode == "add" { i18n.t("barcode.add_to_inventory") } else { i18n.t("barcode.remove_from_inventory") }}
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

            if *is_unknown {
                <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                    <div class="bg-white rounded-xl shadow-xl max-w-md w-full p-6">
                        <h3 class="text-xl font-bold mb-4">{i18n.t("barcode.unknown_title")}</h3>
                        <div class="space-y-4">
                            <p class="text-sm text-gray-500 font-mono bg-gray-50 p-2 rounded">{&*unknown_barcode}</p>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("barcode.product_name")}</label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border rounded-lg outline-none focus:ring-2 focus:ring-blue-500"
                                    value={(*unknown_name).clone()}
                                    oninput={let unknown_name = unknown_name.clone(); Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        unknown_name.set(input.value());
                                    })}
                                    placeholder={i18n.t("barcode.product_name")}
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("barcode.product_brand")}</label>
                                <input
                                    type="text"
                                    class="w-full px-4 py-2 border rounded-lg outline-none focus:ring-2 focus:ring-blue-500"
                                    value={(*unknown_brand).clone()}
                                    oninput={let unknown_brand = unknown_brand.clone(); Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        unknown_brand.set(input.value());
                                    })}
                                    placeholder={i18n.t("barcode.product_brand")}
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("common.unit")}</label>
                                <select
                                    class="w-full px-4 py-2 border rounded-lg outline-none focus:ring-2 focus:ring-blue-500"
                                    value={(*unknown_unit).clone()}
                                    onchange={let unknown_unit = unknown_unit.clone(); Callback::from(move |e: Event| {
                                        let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                        unknown_unit.set(input.value());
                                    })}
                                >
                                    <option value="pcs">{"pcs"}</option>
                                    <option value="kg">{"kg"}</option>
                                    <option value="g">{"g"}</option>
                                    <option value="l">{"l"}</option>
                                    <option value="ml">{"ml"}</option>
                                </select>
                            </div>
                            <div class="flex gap-2 pt-2">
                                <button
                                    onclick={on_submit_unknown}
                                    class="flex-1 py-2 bg-green-600 text-white rounded-lg font-medium hover:bg-green-700 transition"
                                    disabled={(*unknown_name).is_empty() || *loading}
                                >
                                    {i18n.t("common.save")}
                                </button>
                                <button
                                    onclick={on_close_selection.clone()}
                                    class="flex-1 py-2 bg-gray-200 text-gray-700 rounded-lg font-medium hover:bg-gray-300 transition"
                                >
                                    {i18n.t("common.cancel")}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            }
        </div>
    }
}
