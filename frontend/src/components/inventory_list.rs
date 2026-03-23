use crate::api::{
    fetch_inventory, get_inventory_categories, remove_item, update_item,
    InventoryCategory, InventoryItem, RemoveItemRequest, UpdateItemRequest,
};
use crate::app::InventoryContext;
use crate::i18n::use_i18n;
use crate::router::Route;
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::format_quantity;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(InventoryList)]
pub fn inventory_list(_props: &Props) -> Html {
    let items = use_state(|| Vec::<InventoryItem>::new());
    let categories = use_state(|| Vec::<InventoryCategory>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    let i18n = use_i18n();

    // Filter state
    let selected_category_id = use_state(|| None::<String>);
    let filter_expanded = use_state(|| false);

    // Editor state
    let editing_item = use_state(|| None::<InventoryItem>);

    let inventory_context =
        use_context::<InventoryContext>().expect("InventoryContext not found");
    let inventory_id = match &*inventory_context.inventory_id {
        Some(id) => id.clone(),
        None => {
            return html! { <div>{i18n.t("inventory.no_inventory_selected")}</div> };
        }
    };

    let fetch_data = {
        let items = items.clone();
        let categories = categories.clone();
        let loading = loading.clone();
        let error = error.clone();
        let inventory_id = inventory_id.clone();

        Callback::from(move |_: ()| {
            let items = items.clone();
            let categories = categories.clone();
            let loading = loading.clone();
            let error = error.clone();
            let inventory_id = inventory_id.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let items_res = fetch_inventory(&inventory_id).await;
                let cats_res = get_inventory_categories(&inventory_id).await;

                match (items_res, cats_res) {
                    (Ok(inv_items), Ok(inv_cats)) => {
                        items.set(inv_items);
                        categories.set(inv_cats);
                        loading.set(false);
                    }
                    (Err(e), _) | (_, Err(e)) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    {
        let fetch_data = fetch_data.clone();
        use_effect_with((), move |_| {
            fetch_data.emit(());
            || ()
        });
    }

    let navigator = use_navigator().unwrap();
    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::MainMenu);
        })
    };

    // Helper to get all descendant category IDs
    fn get_all_descendants(
        parent_id: &str,
        all_cats: &[InventoryCategory],
    ) -> Vec<String> {
        let mut descendants = vec![parent_id.to_string()];
        for cat in all_cats {
            if cat.parent_id.as_deref() == Some(parent_id) {
                descendants.extend(get_all_descendants(&cat.id, all_cats));
            }
        }
        descendants
    }

    // Filter items based on selected category (including descendants)
    let filtered_items: Vec<_> = items
        .iter()
        .filter(|item| match &*selected_category_id {
            None => true, // All items
            Some(selected_id) if selected_id == "uncategorized" => {
                item.category_ids.is_empty()
            }
            Some(selected_id) => {
                let allowed_ids = get_all_descendants(selected_id, &categories);
                item.category_ids.iter().any(|id| allowed_ids.contains(id))
            }
        })
        .collect();

    let cat_map: HashMap<String, &InventoryCategory> =
        categories.iter().map(|c| (c.id.clone(), c)).collect();

    let on_edit = {
        let editing_item = editing_item.clone();
        move |item: InventoryItem| {
            let editing_item = editing_item.clone();
            Callback::from(move |_| {
                editing_item.set(Some(item.clone()));
            })
        }
    };

    let render_item_card = {
        let i18n = i18n.clone();
        let cat_map = cat_map.clone();
        let on_edit = on_edit.clone();
        let fetch_data = fetch_data.clone();
        let inventory_id = inventory_id.clone();

        move |item: &InventoryItem| {
            let item_clone = item.clone();
            let on_quick_remove = {
                let item_id = item.id.clone();
                let inventory_id = inventory_id.clone();
                let fetch_data = fetch_data.clone();
                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    let item_id = item_id.clone();
                    let inventory_id = inventory_id.clone();
                    let fetch_data = fetch_data.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        let req = RemoveItemRequest {
                            inventory_id,
                            barcode: None,
                            id: Some(item_id),
                            name: None,
                            quantity: Some(1.0),
                        };
                        if let Ok(_) = remove_item(req).await {
                            fetch_data.emit(());
                        }
                    });
                })
            };

            html! {
                <div
                    onclick={on_edit(item_clone.clone())}
                    class="bg-white p-4 rounded-xl shadow-sm border border-gray-100 flex flex-col gap-2 hover:shadow-md transition group cursor-pointer"
                >
                    <div class="flex justify-between items-start">
                        <div class="flex flex-col gap-1 min-w-0">
                            <h3 class="font-semibold text-gray-900 truncate">{&item.name}</h3>
                            {if let Some(ref barcode) = item.barcode {
                                html! { <span class="font-mono text-[10px] text-gray-400">{barcode}</span> }
                            } else {
                                html! {}
                            }}
                        </div>
                        <div class="flex flex-col items-end gap-2">
                            <span class="px-2 py-1 bg-blue-100 text-blue-800 text-xs font-bold rounded-full whitespace-nowrap">
                                {i18n.t_with("inventory.qty", vec![("qty", &format_quantity(item.quantity.into())), ("unit", &item.unit)])}
                            </span>
                            <div class="flex gap-1">
                                <button
                                    onclick={on_quick_remove}
                                    class="p-1.5 text-gray-400 hover:text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                                    title={i18n.t("common.remove_one")}
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M3 10a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1z" clip-rule="evenodd" />
                                    </svg>
                                </button>
                                <button
                                    class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                                    title={i18n.t("common.edit")}
                                >
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                        <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                                    </svg>
                                </button>
                            </div>
                        </div>
                    </div>

                    if !item.category_ids.is_empty() {
                        <div class="flex flex-wrap gap-1 mt-1">
                            { for item.category_ids.iter().filter_map(|id| cat_map.get(id)).map(|cat| html! {
                                <span class="px-2 py-0.5 bg-gray-100 text-gray-600 text-[10px] rounded-md border border-gray-200">
                                    { &cat.name }
                                </span>
                            }) }
                        </div>
                    }
                </div>
            }
        }
    };

    // Helper to render hierarchical filter options
    fn render_filter_tree(
        cats: &[InventoryCategory],
        parent_id: Option<&str>,
        selected_id: &Option<String>,
        on_select: Callback<Option<String>>,
    ) -> Html {
        let level_cats: Vec<_> = cats
            .iter()
            .filter(|c| c.parent_id.as_deref() == parent_id)
            .collect();

        if level_cats.is_empty() {
            return html! {};
        }

        html! {
            <div class="flex flex-col gap-1 ml-4 mt-1 border-l border-gray-100 pl-2">
                { for level_cats.iter().map(|cat| {
                    let id = cat.id.clone();
                    let name = cat.name.clone();
                    let is_selected = selected_id.as_ref() == Some(&id);
                    let on_click = {
                        let id = id.clone();
                        let on_select = on_select.clone();
                        Callback::from(move |_| on_select.emit(Some(id.clone())))
                    };

                    let has_children = cats.iter().any(|c| c.parent_id.as_deref() == Some(&id));

                    html! {
                        <div class="flex flex-col">
                            <button
                                onclick={on_click}
                                class={classes!(
                                    "text-left", "px-3", "py-1.5", "text-sm", "rounded-lg", "transition-all", "flex", "items-center", "gap-2",
                                    if is_selected { "bg-blue-50 text-blue-700 font-semibold" }
                                    else { "text-gray-600 hover:bg-gray-50" }
                                )}
                            >
                                if has_children {
                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-3 w-3 text-gray-400" viewBox="0 0 20 20" fill="currentColor">
                                        <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                                    </svg>
                                } else {
                                    <div class="w-3"></div>
                                }
                                { name }
                            </button>
                            { render_filter_tree(cats, Some(&id), selected_id, on_select.clone()) }
                        </div>
                    }
                }) }
            </div>
        }
    }

    let render_item_editor = {
        let editing_item = editing_item.clone();
        let categories = categories.clone();
        let fetch_data = fetch_data.clone();
        let inventory_id = inventory_id.clone();

        move |item: InventoryItem| {
            let editing_item_close = editing_item.clone();
            let editing_item_success = editing_item.clone();
            let categories = categories.clone();
            let fetch_data = fetch_data.clone();
            let inventory_id = inventory_id.clone();

            html! {
                <ItemEditor
                    inventory_id={inventory_id}
                    item={item}
                    available_categories={(*categories).clone()}
                    on_close={Callback::from(move |_| editing_item_close.set(None))}
                    on_success={Callback::from(move |_| {
                        editing_item_success.set(None);
                        fetch_data.emit(());
                    })}
                />
            }
        }
    };

    html! {
        <div class="max-w-lg mx-auto p-4 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{i18n.t("inventory.title")}</h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{i18n.t("common.back")}</button>
            </div>

            if *loading {
                <div class="flex justify-center p-8">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                </div>
            } else if let Some(ref err) = *error {
                <div class="p-4 bg-red-100 text-red-800 rounded-xl border border-red-200">{i18n.t_with("common.error", vec![("e", err)])}</div>
            } else {
                <>
                    // Category Filter UI
                    <div class="mb-6 bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                        <button
                            onclick={let filter_expanded = filter_expanded.clone(); move |_| filter_expanded.set(!*filter_expanded)}
                            class="w-full px-4 py-3 flex justify-between items-center hover:bg-gray-50 transition-colors"
                        >
                            <div class="flex items-center gap-2">
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-500" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M3 3a1 1 0 011-1h12a1 1 0 011 1v3a1 1 0 01-.293.707L12 11.414V15a1 1 0 01-.293.707l-2 2A1 1 0 018 17v-5.586L3.293 6.707A1 1 0 013 6V3z" clip-rule="evenodd" />
                                </svg>
                                <span class="font-medium text-gray-700">
                                    if let Some(id) = &*selected_category_id {
                                        if id == "uncategorized" {
                                            { i18n.t("inventory.uncategorized") }
                                        } else {
                                            { cat_map.get(id).map(|c| c.name.as_str()).unwrap_or("") }
                                        }
                                    } else {
                                        { i18n.t("category.all_items") }
                                    }
                                </span>
                            </div>
                            <svg xmlns="http://www.w3.org/2000/svg" class={classes!("h-5", "w-5", "text-gray-400", "transition-transform", if *filter_expanded { "rotate-180" } else { "" })} viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" clip-rule="evenodd" />
                            </svg>
                        </button>

                        if *filter_expanded {
                            <div class="px-2 pb-4 pt-1 border-t border-gray-50 max-h-80 overflow-y-auto animate-fade-in">
                                <button
                                    onclick={
                                        let on_select = selected_category_id.clone();
                                        let filter_expanded = filter_expanded.clone();
                                        Callback::from(move |_| {
                                            on_select.set(None);
                                            filter_expanded.set(false);
                                        })
                                    }
                                    class={classes!(
                                        "w-full", "text-left", "px-3", "py-2", "text-sm", "font-medium", "rounded-lg", "transition-all",
                                        if selected_category_id.is_none() { "bg-blue-50 text-blue-700" }
                                        else { "text-gray-600 hover:bg-gray-50" }
                                    )}
                                >
                                    { i18n.t("category.all_items") }
                                </button>

                                { render_filter_tree(&categories, None, &selected_category_id, {
                                    let selected_category_id = selected_category_id.clone();
                                    let filter_expanded = filter_expanded.clone();
                                    Callback::from(move |id| {
                                        selected_category_id.set(id);
                                        filter_expanded.set(false);
                                    })
                                }) }

                                <button
                                    onclick={
                                        let on_select = selected_category_id.clone();
                                        let filter_expanded = filter_expanded.clone();
                                        Callback::from(move |_| {
                                            on_select.set(Some("uncategorized".to_string()));
                                            filter_expanded.set(false);
                                        })
                                    }
                                    class={classes!(
                                        "w-full", "text-left", "px-3", "py-2", "text-sm", "font-medium", "rounded-lg", "transition-all", "mt-1",
                                        if *selected_category_id == Some("uncategorized".to_string()) { "bg-blue-50 text-blue-700" }
                                        else { "text-gray-600 hover:bg-gray-50" }
                                    )}
                                >
                                    { i18n.t("inventory.uncategorized") }
                                </button>
                            </div>
                        }
                    </div>

                    // Items List
                    <div class="space-y-3">
                        { for filtered_items.iter().map(|item| render_item_card(item)) }

                        if filtered_items.is_empty() {
                            <div class="py-12 text-center text-gray-400 italic bg-white rounded-2xl border border-dashed border-gray-200">
                                { i18n.t("inventory.no_items_found") }
                            </div>
                        }
                    </div>

                    if let Some(item) = (*editing_item).clone() {
                        { render_item_editor(item) }
                    }
                </>
            }
        </div>
    }
}

#[derive(Properties, PartialEq)]
struct EditorProps {
    inventory_id: String,
    item: InventoryItem,
    available_categories: Vec<InventoryCategory>,
    on_close: Callback<()>,
    on_success: Callback<()>,
}

#[function_component(ItemEditor)]
fn item_editor(props: &EditorProps) -> Html {
    let i18n = use_i18n();
    let name = use_state(|| props.item.name.clone());
    let quantity = use_state(|| props.item.quantity.to_string());
    let unit = use_state(|| props.item.unit.clone());
    let selected_category_ids = use_state(|| props.item.category_ids.clone());
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);

    let on_submit = {
        let item_id = props.item.id.clone();
        let name = name.clone();
        let quantity = quantity.clone();
        let unit = unit.clone();
        let selected_category_ids = selected_category_ids.clone();
        let loading = loading.clone();
        let error = error.clone();
        let on_success = props.on_success.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let item_id = item_id.clone();
            let name_val = (*name).clone();
            let qty_val = (*quantity).parse::<f32>().unwrap_or(0.0);
            let unit_val = (*unit).clone();
            let cats_val = (*selected_category_ids).clone();
            let loading = loading.clone();
            let error = error.clone();
            let on_success = on_success.clone();

            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let req = UpdateItemRequest {
                    name: Some(name_val),
                    quantity: Some(qty_val),
                    unit: Some(unit_val),
                    categories: Some(cats_val),
                };

                match update_item(&item_id, req).await {
                    Ok(_) => on_success.emit(()),
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    let on_delete = {
        let item_id = props.item.id.clone();
        let inventory_id = props.inventory_id.clone();
        let current_qty = props.item.quantity;
        let loading = loading.clone();
        let error = error.clone();
        let on_success = props.on_success.clone();

        Callback::from(move |_| {
            let item_id = item_id.clone();
            let inventory_id = inventory_id.clone();
            let loading = loading.clone();
            let error = error.clone();
            let on_success = on_success.clone();

            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                let req = RemoveItemRequest {
                    inventory_id,
                    barcode: None,
                    id: Some(item_id),
                    name: None,
                    quantity: Some(current_qty),
                };

                match remove_item(req).await {
                    Ok(_) => on_success.emit(()),
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50 animate-fade-in">
            <div class="bg-white rounded-2xl shadow-xl w-full max-w-md overflow-hidden animate-slide-up">
                <div class="p-6 border-b border-gray-100 flex justify-between items-center bg-gray-50">
                    <h2 class="text-xl font-bold text-gray-800">{ i18n.t("inventory.edit_item") }</h2>
                    <button onclick={let on_close = props.on_close.clone(); move |_| on_close.emit(())} class="text-gray-400 hover:text-gray-600">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                </div>

                <form onsubmit={on_submit} class="p-6 space-y-4">
                    if let Some(e) = &*error {
                        <div class="p-3 bg-red-100 text-red-800 text-sm rounded-lg border border-red-200">{ e }</div>
                    }

                    <div class="space-y-1">
                        <label class="text-sm font-medium text-gray-700">{ i18n.t("barcode.product_name") }</label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none"
                            value={ (*name).clone() }
                            oninput={let name = name.clone(); Callback::from(move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                name.set(input.value());
                            })}
                        />
                    </div>

                    <div class="grid grid-cols-2 gap-4">
                        <div class="space-y-1">
                            <label class="text-sm font-medium text-gray-700">{ i18n.t("common.qty_label") }</label>
                            <input
                                type="number"
                                step="0.01"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none"
                                value={ (*quantity).clone() }
                                oninput={let quantity = quantity.clone(); Callback::from(move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    quantity.set(input.value());
                                })}
                            />
                        </div>
                        <div class="space-y-1">
                            <label class="text-sm font-medium text-gray-700">{ i18n.t("common.unit") }</label>
                            <select
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none bg-white"
                                onchange={let unit = unit.clone(); Callback::from(move |e: Event| {
                                    let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                    unit.set(input.value());
                                })}
                            >
                                <option value="pcs" selected={*unit == "pcs"}>{ i18n.t("custom_items.unit_pcs") }</option>
                                <option value="kg" selected={*unit == "kg"}>{ i18n.t("custom_items.unit_kg") }</option>
                                <option value="g" selected={*unit == "g"}>{ i18n.t("custom_items.unit_g") }</option>
                                <option value="l" selected={*unit == "l"}>{ i18n.t("custom_items.unit_l") }</option>
                                <option value="ml" selected={*unit == "ml"}>{ i18n.t("custom_items.unit_ml") }</option>
                            </select>
                        </div>
                    </div>

                    <div class="space-y-2">
                        <label class="text-sm font-medium text-gray-700 block">{ i18n.t("category.title") }</label>
                        <div class="flex flex-wrap gap-2 max-h-40 overflow-y-auto p-3 border border-gray-200 rounded-xl bg-gray-50">
                            { for props.available_categories.iter().map(|cat| {
                                let cat_id = cat.id.clone();
                                let is_selected = (*selected_category_ids).contains(&cat_id);
                                let on_toggle = {
                                    let cat_id = cat_id.clone();
                                    let selected_ids = selected_category_ids.clone();
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
                                        type="button"
                                        onclick={on_toggle}
                                        class={classes!(
                                            "px-3", "py-1", "text-xs", "rounded-full", "border", "transition-all",
                                            if is_selected { "bg-blue-600 text-white border-blue-600 shadow-sm" }
                                            else { "bg-white text-gray-600 border-gray-200 hover:border-blue-300" }
                                        )}
                                    >
                                        { &cat.name }
                                    </button>
                                }
                            }) }
                        </div>
                    </div>

                    <div class="flex flex-col sm:flex-row gap-3 pt-4 border-t border-gray-100">
                        <button
                            type="button"
                            onclick={on_delete}
                            disabled={*loading}
                            class="px-4 py-2 bg-red-50 text-red-600 rounded-lg font-medium hover:bg-red-100 transition flex items-center justify-center gap-2"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                            </svg>
                            { i18n.t("common.delete") }
                        </button>
                        <div class="flex-1 flex gap-3">
                            <button
                                type="button"
                                onclick={let on_close = props.on_close.clone(); move |_| on_close.emit(())}
                                class="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg font-medium hover:bg-gray-50 transition"
                            >
                                { i18n.t("common.cancel") }
                            </button>
                            <button
                                type="submit"
                                disabled={*loading}
                                class="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition disabled:opacity-50 flex items-center justify-center gap-2"
                            >
                                if *loading {
                                    <div class="animate-spin h-4 w-4 border-2 border-white border-t-transparent rounded-full"></div>
                                }
                                { i18n.t("common.save") }
                            </button>
                        </div>
                    </div>
                </form>
            </div>
        </div>
    }
}
