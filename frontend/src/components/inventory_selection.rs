use crate::api::{
    create_inventory, get_user_inventories, CreateInventoryRequest, Inventory,
};
use crate::app::{InventoryContext, UserContext};
use crate::i18n::use_i18n;
use crate::router::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(InventorySelection)]
pub fn inventory_selection(_props: &Props) -> Html {
    let inventories = use_state(|| Vec::<Inventory>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    let show_create = use_state(|| false);
    let editing_inv = use_state(|| None::<Inventory>);
    let new_inv_name = use_state(|| String::new());
    let new_inv_lang = use_state(|| "en".to_string());
    let i18n = use_i18n();

    let user_context =
        use_context::<UserContext>().expect("UserContext not found");
    let inventory_context =
        use_context::<InventoryContext>().expect("InventoryContext not found");
    let navigator = use_navigator().unwrap();

    let user_id = match &*user_context.user {
        Some(user) => user.id.clone(),
        None => {
            // Or redirect to login
            return html! { <div>{i18n.t("login.please_log_in")}</div> };
        }
    };

    {
        let inventories = inventories.clone();
        let loading = loading.clone();
        let error = error.clone();
        let user_id = user_id.clone();
        let inventory_context = inventory_context.clone();
        let navigator = navigator.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match get_user_inventories(&user_id).await {
                    Ok(invs) => {
                        // Check if we already have an inventory_id set and it exists in user inventories
                        let current_id =
                            (*inventory_context.inventory_id).clone();
                        if let Some(id) = current_id {
                            if invs.iter().any(|inv| inv.id == id) {
                                navigator.push(&Route::MainMenu);
                                return;
                            }
                        }

                        inventories.set(invs);
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

    let on_create = {
        let inventories = inventories.clone();
        let new_inv_name = new_inv_name.clone();
        let new_inv_lang = new_inv_lang.clone();
        let show_create = show_create.clone();
        let user_id = user_id.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let name = (*new_inv_name).clone();
            let category_language = Some((*new_inv_lang).clone());
            if name.trim().is_empty() {
                return;
            }

            let inventories = inventories.clone();
            let show_create = show_create.clone();
            let new_inv_name = new_inv_name.clone();
            let new_inv_lang = new_inv_lang.clone();
            let user_id = user_id.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match create_inventory(CreateInventoryRequest {
                    name,
                    owner_id: user_id,
                    category_language,
                })
                .await
                {
                    Ok(inv) => {
                        let mut current = (*inventories).clone();
                        current.push(inv);
                        inventories.set(current);
                        show_create.set(false);
                        new_inv_name.set(String::new());
                        new_inv_lang.set("en".to_string());
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_update = {
        let inventories = inventories.clone();
        let editing_inv = editing_inv.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let inv = match &*editing_inv {
                Some(i) => i.clone(),
                None => return,
            };

            let inventories = inventories.clone();
            let editing_inv = editing_inv.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                use crate::api::{update_inventory, UpdateInventoryRequest};
                match update_inventory(
                    &inv.id,
                    UpdateInventoryRequest {
                        name: Some(inv.name.clone()),
                        category_language: Some(inv.category_language.clone()),
                    },
                )
                .await
                {
                    Ok(_) => {
                        let mut current = (*inventories).clone();
                        if let Some(idx) =
                            current.iter().position(|i| i.id == inv.id)
                        {
                            current[idx] = inv;
                        }
                        inventories.set(current);
                        editing_inv.set(None);
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let languages = vec![
        ("en", "English"),
        ("fr", "Français"),
        ("cs", "Čeština"),
        ("de", "Deutsch"),
        ("es", "Español"),
        ("it", "Italiano"),
        ("pl", "Polski"),
    ];

    html! {
        <div class="min-h-screen bg-gray-50 p-4">
            <div class="max-w-md mx-auto">
                <div class="flex justify-between items-center mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">{i18n.t("inventory.selection_title")}</h1>
                    <button
                        onclick={
                            let user_context = user_context.clone();
                            let inventory_context = inventory_context.clone();
                            let navigator = navigator.clone();
                            Callback::from(move |_| {
                                user_context.user.set(None);
                                inventory_context.inventory_id.set(None);
                                navigator.push(&Route::Login);
                            })
                        }
                        class="p-2 text-gray-500 hover:text-red-600 transition flex items-center gap-1 text-sm font-medium"
                        title={i18n.t("main_menu.logout")}
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                        </svg>
                        {i18n.t("main_menu.logout")}
                    </button>
                </div>

                if let Some(ref err) = *error {
                    <div class="mb-4 p-3 bg-red-100 text-red-700 rounded-lg text-sm">{i18n.t_with("common.error", vec![("e", err)])}</div>
                }

                if *loading {
                    <div class="flex justify-center p-8">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                } else {
                    <div class="space-y-3">
                        {for inventories.iter().map(|inv| {
                            let inventory_context = inventory_context.clone();
                            let editing_inv = editing_inv.clone();
                            let navigator = navigator.clone();
                            let id = inv.id.clone();
                            let i18n = i18n.clone();

                            let on_select = {
                                let id = id.clone();
                                Callback::from({let navigator = navigator.clone(); move |_| {
                                    inventory_context.inventory_id.set(Some(id.clone()));
                                    navigator.push(&Route::MainMenu);
                                }})
                            };

                            let on_share = {
                                let navigator = navigator.clone();
                                let id = id.clone();
                                Callback::from(move |_| {
                                    navigator.push(&Route::Share { id: id.clone() });
                                })
                            };

                            let on_categories = {
                                let navigator = navigator.clone();
                                let id = id.clone();
                                Callback::from(move |_| {
                                    navigator.push(&Route::Categories { id: id.clone() });
                                })
                            };

                            let on_edit = {
                                let editing_inv = editing_inv.clone();
                                let inv = inv.clone();
                                Callback::from(move |_| {
                                    editing_inv.set(Some(inv.clone()));
                                })
                            };

                            html! {
                                <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200 transition flex flex-col sm:flex-row justify-between sm:items-center gap-4">
                                    <div class="flex flex-col min-w-0">
                                        <span class="font-medium text-gray-800 truncate" title={inv.name.clone()}>{&inv.name}</span>
                                        <span class="text-xs text-gray-500 uppercase">{&inv.category_language}</span>
                                    </div>
                                    <div class="flex flex-wrap gap-2">
                                        <button onclick={on_categories} class="p-2 sm:px-2 sm:py-1 bg-gray-100 text-gray-600 rounded-md hover:bg-gray-200" title={i18n.t("Categories")}>
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                <path d="M7 3a1 1 0 000 2h6a1 1 0 100-2H7zM4 7a1 1 0 011-1h10a1 1 0 110 2H5a1 1 0 01-1-1zM2 11a2 2 0 012-2h12a2 2 0 012 2v4a2 2 0 01-2 2H4a2 2 0 01-2-2v-4z" />
                                            </svg>
                                        </button>
                                        <button onclick={on_edit} class="p-2 sm:px-2 sm:py-1 bg-gray-100 text-gray-600 rounded-md hover:bg-gray-200" title={i18n.t("common.edit")}>
                                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.38-8.379-2.83-2.828z" />
                                            </svg>
                                        </button>
                                        <button onclick={on_share} class="flex-1 sm:flex-none px-3 py-1 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300 text-sm">{i18n.t("inventory.share")}</button>
                                        <button onclick={on_select} class="flex-1 sm:flex-none px-3 py-1 bg-blue-600 text-white rounded-md hover:bg-blue-700 text-sm font-medium">{i18n.t("inventory.switch")}</button>
                                    </div>
                                </div>
                            }
                        })}

                        if let Some(inv) = &*editing_inv {
                            <form onsubmit={on_update} class="bg-white p-4 rounded-xl shadow-sm border border-blue-200 mt-4">
                                <h3 class="font-medium text-gray-900 mb-3">{i18n.t("inventory.edit_title")}</h3>
                                <div class="space-y-3">
                                    <div>
                                        <label class="text-xs font-semibold text-gray-500 uppercase ml-1">{i18n.t("inventory.name_label")}</label>
                                        <input
                                            type="text"
                                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none"
                                            value={inv.name.clone()}
                                            oninput={
                                                let editing_inv = editing_inv.clone();
                                                let inv = inv.clone();
                                                Callback::from(move |e: InputEvent| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    let mut new_inv = inv.clone();
                                                    new_inv.name = input.value();
                                                    editing_inv.set(Some(new_inv));
                                                })
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label class="text-xs font-semibold text-gray-500 uppercase ml-1">{i18n.t("inventory.category_language")}</label>
                                        <select
                                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none bg-white"
                                            value={inv.category_language.clone()}
                                            onchange={
                                                let editing_inv = editing_inv.clone();
                                                let inv = inv.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    let mut new_inv = inv.clone();
                                                    new_inv.category_language = input.value();
                                                    editing_inv.set(Some(new_inv));
                                                })
                                            }
                                        >
                                            {for languages.iter().map(|(code, name)| {
                                                html! { <option value={*code} selected={inv.category_language == *code}>{name}</option> }
                                            })}
                                        </select>
                                    </div>
                                </div>
                                <div class="flex flex-col sm:flex-row gap-2 mt-4">
                                    <button type="submit" class="w-full sm:flex-1 bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700 transition">{i18n.t("common.save")}</button>
                                    <button
                                        type="button"
                                        class="w-full sm:flex-1 bg-gray-100 text-gray-700 py-2 rounded-lg hover:bg-gray-200 transition"
                                        onclick={Callback::from(move |_| editing_inv.set(None))}
                                    >
                                        {i18n.t("common.cancel")}
                                    </button>
                                </div>
                            </form>
                        } else if *show_create {
                            <form onsubmit={on_create} class="bg-white p-4 rounded-xl shadow-sm border border-gray-200 mt-4">
                                <h3 class="font-medium text-gray-900 mb-3">{i18n.t("inventory.create_new")}</h3>
                                <div class="space-y-3">
                                    <input
                                        type="text"
                                        class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none"
                                        placeholder={i18n.t("inventory.name_placeholder")}
                                        value={(*new_inv_name).clone()}
                                        oninput={Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            new_inv_name.set(input.value());
                                        })}
                                    />
                                    <div>
                                        <label class="text-xs font-semibold text-gray-500 uppercase ml-1">{i18n.t("inventory.category_language")}</label>
                                        <select
                                            class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none bg-white"
                                            value={(*new_inv_lang).clone()}
                                            onchange={
                                                let new_inv_lang = new_inv_lang.clone();
                                                Callback::from(move |e: Event| {
                                                    let input: HtmlInputElement = e.target_unchecked_into();
                                                    new_inv_lang.set(input.value());
                                                })
                                            }
                                        >
                                            {for languages.iter().map(|(code, name)| {
                                                html! { <option value={*code} selected={*new_inv_lang == *code}>{name}</option> }
                                            })}
                                        </select>
                                    </div>
                                </div>
                                <div class="flex flex-col sm:flex-row gap-2 mt-4">
                                    <button type="submit" class="w-full sm:flex-1 bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700 transition">{i18n.t("inventory.create_button")}</button>
                                    <button
                                        type="button"
                                        class="w-full sm:flex-1 bg-gray-100 text-gray-700 py-2 rounded-lg hover:bg-gray-200 transition"
                                        onclick={Callback::from(move |_| show_create.set(false))}
                                    >
                                        {i18n.t("common.cancel")}
                                    </button>
                                </div>
                            </form>
                        } else {
                            <button
                                class="w-full py-3 border-2 border-dashed border-gray-300 rounded-xl text-gray-500 hover:border-blue-500 hover:text-blue-600 transition font-medium flex items-center justify-center gap-2"
                                onclick={Callback::from(move |_| show_create.set(true))}
                            >
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                    <path fill-rule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clip-rule="evenodd" />
                                </svg>
                                {i18n.t("inventory.create_new")}
                            </button>
                        }
                    </div>
                }
            </div>
        </div>
    }
}
