use crate::api::{
    fetch_inventory, get_inventory_categories, update_item, InventoryCategory,
    InventoryItem, UpdateItemRequest,
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
        move |item: &InventoryItem| {
            let item_clone = item.clone();
            html! {
                <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-100 flex flex-col gap-2 hover:shadow-md transition group">
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
                            <button
                                onclick={on_edit(item_clone)}
                                class="p-1.5 text-gray-400 hover:text-blue-600 hover:bg-blue-50 rounded-lg transition-colors"
                                title={i18n.t("common.edit")}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                                    <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                                </svg>
                            </button>
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
    fn render_filter_options(
        cats: &[InventoryCategory],
        parent_id: Option<&str>,
        depth: usize,
        selected_id: &Option<String>,
        on_select: Callback<Option<String>>,
    ) -> Vec<Html> {
        let mut options = Vec::new();
        let level_cats: Vec<_> = cats
            .iter()
            .filter(|c| c.parent_id.as_deref() == parent_id)
            .collect();

        for cat in level_cats {
            let id = cat.id.clone();
            let name = cat.name.clone();
            let is_selected = selected_id.as_ref() == Some(&id);
            let on_click = {
                let id = id.clone();
                let on_select = on_select.clone();
                Callback::from(move |_| on_select.emit(Some(id.clone())))
            };

            options.push(html! {
                <button
                    onclick={on_click}
                    class={classes!(
                        "px-3", "py-1.5", "text-sm", "rounded-full", "border", "transition-all", "whitespace-nowrap",
                        if is_selected { "bg-blue-600 text-white border-blue-600 shadow-sm scale-105" }
                        else { "bg-white text-gray-600 border-gray-200 hover:border-blue-300" }
                    )}
                    style={format!("margin-left: {}px", depth * 8)}
                >
                    { name }
                </button>
            });

            options.extend(render_filter_options(
                cats,
                Some(&id),
                depth + 1,
                selected_id,
                on_select.clone(),
            ));
        }
        options
    }

    let render_item_editor = {
        let editing_item = editing_item.clone();
        let categories = categories.clone();
        let fetch_data = fetch_data.clone();

        move |item: InventoryItem| {
            let editing_item_close = editing_item.clone();
            let editing_item_success = editing_item.clone();
            let categories = categories.clone();
            let fetch_data = fetch_data.clone();

            html! {
                <ItemEditor
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
                    <div class="mb-8 overflow-x-auto pb-4 no-scrollbar">
                        <div class="flex gap-2 min-w-max px-1">
                            <button
                                onclick={
                                    let on_select = selected_category_id.clone();
                                    Callback::from(move |_| on_select.set(None))
                                }
                                class={classes!(
                                    "px-4", "py-1.5", "text-sm", "font-medium", "rounded-full", "border", "transition-all",
                                    if selected_category_id.is_none() { "bg-gray-800 text-white border-gray-800 shadow-md scale-105" }
                                    else { "bg-white text-gray-600 border-gray-200 hover:border-gray-300" }
                                )}
                            >
                                { i18n.t("category.all_items") }
                            </button>

                            { for render_filter_options(&categories, None, 0, &selected_category_id, {
                                let selected_category_id = selected_category_id.clone();
                                Callback::from(move |id| selected_category_id.set(id))
                            }) }

                            <button
                                onclick={
                                    let on_select = selected_category_id.clone();
                                    Callback::from(move |_| on_select.set(Some("uncategorized".to_string())))
                                }
                                class={classes!(
                                    "px-4", "py-1.5", "text-sm", "font-medium", "rounded-full", "border", "transition-all",
                                    if *selected_category_id == Some("uncategorized".to_string()) { "bg-gray-500 text-white border-gray-500 shadow-md scale-105" }
                                    else { "bg-white text-gray-400 border-gray-200 border-dashed hover:border-gray-300" }
                                )}
                            >
                                { i18n.t("inventory.uncategorized") }
                            </button>
                        </div>
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
                                value={ (*unit).clone() }
                                onchange={let unit = unit.clone(); Callback::from(move |e: Event| {
                                    let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                    unit.set(input.value());
                                })}
                            >
                                <option value="pcs">{ i18n.t("custom_items.unit_pcs") }</option>
                                <option value="kg">{ i18n.t("custom_items.unit_kg") }</option>
                                <option value="g">{ i18n.t("custom_items.unit_g") }</option>
                                <option value="l">{ i18n.t("custom_items.unit_l") }</option>
                                <option value="ml">{ i18n.t("custom_items.unit_ml") }</option>
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

                    <div class="flex gap-3 pt-4 border-t border-gray-100">
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
                </form>
            </div>
        </div>
    }
}
