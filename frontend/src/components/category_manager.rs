use crate::api::{
    create_inventory_category, delete_inventory_category,
    get_inventory_categories, update_inventory_category, CreateCategoryRequest,
    InventoryCategory, UpdateCategoryRequest,
};
use crate::i18n::use_i18n;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub inventory_id: String,
}

#[function_component(CategoryManager)]
pub fn category_manager(props: &Props) -> Html {
    let i18n = use_i18n();
    let categories = use_state(|| Vec::<InventoryCategory>::new());
    let error = use_state(|| None::<String>);
    let new_category_name = use_state(|| String::new());
    let new_category_parent = use_state(|| None::<String>);
    let editing_id = use_state(|| None::<String>);
    let editing_name = use_state(|| String::new());

    let load_categories = {
        let categories = categories.clone();
        let error = error.clone();
        let inventory_id = props.inventory_id.clone();
        Callback::from(move |_| {
            let categories = categories.clone();
            let error = error.clone();
            let inventory_id = inventory_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match get_inventory_categories(&inventory_id).await {
                    Ok(cats) => categories.set(cats),
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    {
        let load_categories = load_categories.clone();
        use_effect_with((), move |_| {
            load_categories.emit(());
            || ()
        });
    }

    let on_add = {
        let inventory_id = props.inventory_id.clone();
        let name_state = new_category_name.clone();
        let parent_state = new_category_parent.clone();
        let load_categories = load_categories.clone();
        let error = error.clone();
        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let inventory_id = inventory_id.clone();
            let name = (*name_state).clone();
            let parent_id = (*parent_state).clone();
            let name_state = name_state.clone();
            let load_categories = load_categories.clone();
            let error = error.clone();

            if name.is_empty() {
                return;
            }

            wasm_bindgen_futures::spawn_local(async move {
                let req = CreateCategoryRequest { name, parent_id };
                match create_inventory_category(&inventory_id, req).await {
                    Ok(_) => {
                        name_state.set(String::new());
                        load_categories.emit(());
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_delete = {
        let inventory_id = props.inventory_id.clone();
        let load_categories = load_categories.clone();
        let error = error.clone();
        move |category_id: String| {
            let inventory_id = inventory_id.clone();
            let load_categories = load_categories.clone();
            let error = error.clone();
            Callback::from(move |_| {
                let inventory_id = inventory_id.clone();
                let category_id = category_id.clone();
                let load_categories = load_categories.clone();
                let error = error.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match delete_inventory_category(&inventory_id, &category_id)
                        .await
                    {
                        Ok(_) => load_categories.emit(()),
                        Err(e) => error.set(Some(e)),
                    }
                });
            })
        }
    };

    let on_save_rename = {
        let inventory_id = props.inventory_id.clone();
        let load_categories = load_categories.clone();
        let error = error.clone();
        let editing_id = editing_id.clone();
        let editing_name = editing_name.clone();

        Callback::from(move |_| {
            let inventory_id = inventory_id.clone();
            let load_categories = load_categories.clone();
            let error = error.clone();
            let editing_id = editing_id.clone();
            let editing_name = editing_name.clone();

            let cat_id = match &*editing_id {
                Some(id) => id.clone(),
                None => return,
            };
            let name = (*editing_name).clone();

            wasm_bindgen_futures::spawn_local(async move {
                let req = UpdateCategoryRequest {
                    name: Some(name),
                    parent_id: None,
                };
                match update_inventory_category(&inventory_id, &cat_id, req)
                    .await
                {
                    Ok(_) => {
                        editing_id.set(None);
                        load_categories.emit(());
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_update_parent = {
        let inventory_id = props.inventory_id.clone();
        let load_categories = load_categories.clone();
        let error = error.clone();
        move |category_id: String| {
            let inventory_id = inventory_id.clone();
            let load_categories = load_categories.clone();
            let error = error.clone();
            Callback::from(move |e: Event| {
                let input: web_sys::HtmlSelectElement =
                    e.target_unchecked_into();
                let new_parent_id = if input.value().is_empty() {
                    None
                } else {
                    Some(input.value())
                };

                let inventory_id = inventory_id.clone();
                let category_id = category_id.clone();
                let load_categories = load_categories.clone();
                let error = error.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let req = UpdateCategoryRequest {
                        name: None,
                        parent_id: Some(new_parent_id),
                    };
                    match update_inventory_category(
                        &inventory_id,
                        &category_id,
                        req,
                    )
                    .await
                    {
                        Ok(_) => load_categories.emit(()),
                        Err(e) => error.set(Some(e)),
                    }
                });
            })
        }
    };

    html! {
        <div class="max-w-2xl mx-auto p-4 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{ i18n.t("category.title") }</h1>
                <button
                    onclick={|_| {
                        let window = web_sys::window().unwrap();
                        window.history().unwrap().back().unwrap();
                    }}
                    class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium"
                >
                    { i18n.t("common.back") }
                </button>
            </div>

            if let Some(e) = &*error {
                <div class="mb-4 p-4 bg-red-100 text-red-800 rounded-xl border border-red-200">{ e }</div>
            }

            <div class="mb-8 bg-white p-5 rounded-xl shadow-sm border border-gray-100">
                <h2 class="text-lg font-semibold mb-4 text-gray-700">{ i18n.t("category.new_category") }</h2>
                <form onsubmit={on_add} class="flex flex-col md:flex-row gap-3">
                    <input
                        type="text"
                        placeholder={ i18n.t("category.name_placeholder") }
                        class="flex-[2] min-w-0 px-4 py-2 border border-gray-300 rounded-lg outline-none focus:ring-2 focus:ring-blue-500"
                        value={ (*new_category_name).clone() }
                        oninput={
                            let name = new_category_name.clone();
                            Callback::from(move |e: InputEvent| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                name.set(input.value());
                            })
                        }
                    />
                    <select
                        class="flex-1 min-w-0 px-4 py-2 border border-gray-300 rounded-lg outline-none focus:ring-2 focus:ring-blue-500 bg-white"
                        onchange={
                            let parent = new_category_parent.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                let val = input.value();
                                parent.set(if val.is_empty() { None } else { Some(val) });
                            })
                        }
                    >
                        <option value="">{ i18n.t("category.no_parent") }</option>
                        { for categories.iter().map(|cat| html! {
                            <option value={cat.id.clone()}>{ &cat.name }</option>
                        }) }
                    </select>
                    <button
                        type="submit"
                        class="w-full md:w-auto flex-shrink-0 px-6 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition"
                    >
                        { i18n.t("category.add") }
                    </button>
                </form>
            </div>

            <div class="bg-white rounded-xl shadow-sm border border-gray-100 overflow-x-auto">
                <table class="w-full text-left border-collapse min-w-[500px]">
                    <thead>
                        <tr class="bg-gray-50 border-b border-gray-100">
                            <th class="p-4 font-semibold text-gray-700 w-1/3">{ i18n.t("category.name") }</th>
                            <th class="p-4 font-semibold text-gray-700 w-1/3">{ i18n.t("category.parent") }</th>
                            <th class="p-4 font-semibold text-gray-700 text-right w-1/3">{ i18n.t("category.actions") }</th>
                        </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-50">
                        { for categories.iter().map(|cat| {
                            let cat_id = cat.id.clone();
                            let is_editing = *editing_id == Some(cat.id.clone());

                            html! {
                                <tr key={cat.id.clone()} class="hover:bg-gray-50 transition-colors">
                                    <td class="p-4 text-gray-900 font-medium">
                                        if is_editing {
                                            <input
                                                type="text"
                                                class="w-full px-2 py-1 border border-blue-300 rounded outline-none focus:ring-2 focus:ring-blue-500"
                                                value={ (*editing_name).clone() }
                                                oninput={
                                                    let editing_name = editing_name.clone();
                                                    Callback::from(move |e: InputEvent| {
                                                        let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                                        editing_name.set(input.value());
                                                    })
                                                }
                                            />
                                        } else {
                                            <div class="truncate max-w-[150px] sm:max-w-none" title={cat.name.clone()}>
                                                { &cat.name }
                                            </div>
                                        }
                                    </td>
                                    <td class="p-4">
                                        <select
                                            class="w-full text-sm px-2 py-1 border border-gray-200 rounded outline-none focus:ring-2 focus:ring-blue-500 bg-transparent"
                                            onchange={ on_update_parent(cat.id.clone()) }
                                        >
                                            <option value="" selected={ cat.parent_id.is_none() }>{ i18n.t("category.no_parent") }</option>
                                            { for categories.iter().filter(|c| c.id != cat.id).map(|c| {
                                                let is_selected = cat.parent_id.as_ref() == Some(&c.id);
                                                html! {
                                                    <option value={c.id.clone()} selected={is_selected}>{ &c.name }</option>
                                                }
                                            }) }
                                        </select>
                                    </td>
                                    <td class="p-4 text-right">
                                        if is_editing {
                                            <div class="flex flex-wrap justify-end gap-1 sm:gap-2">
                                                <button
                                                    onclick={ on_save_rename.clone() }
                                                    class="px-2 sm:px-3 py-1 bg-green-50 text-green-600 rounded-md hover:bg-green-100 transition-colors text-xs sm:text-sm font-medium"
                                                >
                                                    { i18n.t("common.save") }
                                                </button>
                                                <button
                                                    onclick={
                                                        let editing_id = editing_id.clone();
                                                        Callback::from(move |_| editing_id.set(None))
                                                    }
                                                    class="px-2 sm:px-3 py-1 bg-gray-50 text-gray-600 rounded-md hover:bg-gray-100 transition-colors text-xs sm:text-sm font-medium"
                                                >
                                                    { i18n.t("common.cancel") }
                                                </button>
                                            </div>
                                        } else {
                                            <div class="flex flex-wrap justify-end gap-1 sm:gap-2">
                                                <button
                                                    onclick={
                                                        let editing_id = editing_id.clone();
                                                        let editing_name = editing_name.clone();
                                                        let cat = cat.clone();
                                                        Callback::from(move |_| {
                                                            editing_id.set(Some(cat.id.clone()));
                                                            editing_name.set(cat.name.clone());
                                                        })
                                                    }
                                                    class="p-1 sm:px-3 sm:py-1 bg-blue-50 text-blue-600 rounded-md hover:bg-blue-100 transition-colors text-xs sm:text-sm font-medium"
                                                    title={i18n.t("common.edit")}
                                                >
                                                    <span class="hidden sm:inline">{ i18n.t("common.edit") }</span>
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 sm:hidden" viewBox="0 0 20 20" fill="currentColor">
                                                        <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                                                    </svg>
                                                </button>
                                                <button
                                                    onclick={ on_delete(cat_id) }
                                                    class="p-1 sm:px-3 sm:py-1 bg-red-50 text-red-600 rounded-md hover:bg-red-100 transition-colors text-xs sm:text-sm font-medium"
                                                    title={i18n.t("common.delete")}
                                                >
                                                    <span class="hidden sm:inline">{ i18n.t("common.delete") }</span>
                                                    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 sm:hidden" viewBox="0 0 20 20" fill="currentColor">
                                                        <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                                                    </svg>
                                                </button>
                                            </div>
                                        }
                                    </td>
                                </tr>
                            }
                        }) }
                    </tbody>
                </table>
                if categories.is_empty() {
                    <div class="p-8 text-center text-gray-500 italic">
                        { i18n.t("category.no_found") }
                    </div>
                }
            </div>
        </div>
    }
}
