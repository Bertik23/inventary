use crate::router::Route;
use crate::app::{UserContext, InventoryContext};
use crate::i18n::{use_i18n, Language};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(MainMenu)]
pub fn main_menu(_props: &Props) -> Html {
    let navigator = use_navigator().unwrap();
    let user_context = use_context::<UserContext>().expect("UserContext not found");
    let inventory_context = use_context::<InventoryContext>().expect("InventoryContext not found");
    let i18n = use_i18n();

    let set_lang = {
        let language = i18n.language.clone();
        Callback::from(move |lang: Language| {
            language.set(lang);
        })
    };

    let on_add_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Add);
        })
    };

    let on_remove_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Remove);
        })
    };

    let on_show_inventory_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Inventory);
        })
    };

    let on_share_click = {
        let navigator = navigator.clone();
        let inventory_id = (*inventory_context.inventory_id).clone();
        Callback::from(move |_| {
            if let Some(ref id) = inventory_id {
                navigator.push(&Route::Share { id: id.clone() });
            }
        })
    };

    let on_manage_inventories_click = {
        let navigator = navigator.clone();
        let inventory_context = inventory_context.clone();
        Callback::from(move |_| {
            inventory_context.inventory_id.set(None);
            navigator.push(&Route::Selection);
        })
    };

    let on_account_settings_click = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Profile);
        })
    };

    let on_manage_custom_items_click = {
        let navigator = navigator.clone();
        let inventory_id = (*inventory_context.inventory_id).clone();
        Callback::from(move |_| {
            if let Some(ref id) = inventory_id {
                navigator.push(&Route::CustomItems { id: id.clone() });
            }
        })
    };

    let on_logout_click = {
        let user_context = user_context.clone();
        let inventory_context = inventory_context.clone();
        let navigator = navigator.clone();
        Callback::from(move |_| {
            user_context.user.set(None);
            inventory_context.inventory_id.set(None);
            navigator.push(&Route::Login);
        })
    };

    html! {
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-4">
            <div class="w-full flex justify-end gap-2 mb-4 max-w-md">
                <button 
                    onclick={let set_lang = set_lang.clone(); move |_| set_lang.emit(Language::En)}
                    class={classes!("px-2", "py-1", "text-xs", "rounded", "border", "transition", if *i18n.language == Language::En { "bg-blue-600 text-white border-blue-600" } else { "bg-white text-gray-600 border-gray-200" })}
                >
                    {"EN"}
                </button>
                <button 
                    onclick={let set_lang = set_lang.clone(); move |_| set_lang.emit(Language::Cs)}
                    class={classes!("px-2", "py-1", "text-xs", "rounded", "border", "transition", if *i18n.language == Language::Cs { "bg-blue-600 text-white border-blue-600" } else { "bg-white text-gray-600 border-gray-200" })}
                >
                    {"CS"}
                </button>
            </div>
            <div class="w-full max-w-md space-y-8">
                <div class="text-center">
                    <div class="mx-auto h-20 w-20 bg-blue-600 rounded-2xl flex items-center justify-center shadow-lg mb-6 transform -rotate-3">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                        </svg>
                    </div>
                    <h1 class="text-3xl font-extrabold text-gray-900 tracking-tight">{i18n.t("main_menu.title")}</h1>
                    <p class="mt-2 text-gray-500">{if let Some(user) = &*user_context.user { user.username.clone() } else { i18n.t("common.guest") }}</p>
                </div>

                <div class="space-y-4">
                    <button
                        onclick={on_add_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-blue-300 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-blue-50 text-blue-600 rounded-lg flex items-center justify-center group-hover:bg-blue-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.add_item")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("main_menu.add_desc")}</p>
                        </div>
                    </button>

                    <button
                        onclick={on_remove_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-red-300 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-red-50 text-red-600 rounded-lg flex items-center justify-center group-hover:bg-red-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 12H4" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.remove_item")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("main_menu.remove_desc")}</p>
                        </div>
                    </button>

                    <button
                        onclick={on_show_inventory_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-indigo-300 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-indigo-50 text-indigo-600 rounded-lg flex items-center justify-center group-hover:bg-indigo-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.view_inventory")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("main_menu.view_desc")}</p>
                        </div>
                    </button>

                    <button
                        onclick={on_share_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-teal-300 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-teal-50 text-teal-600 rounded-lg flex items-center justify-center group-hover:bg-teal-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.share_inventory")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("main_menu.share_desc")}</p>
                        </div>
                    </button>

                    <button
                        onclick={on_manage_custom_items_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-purple-300 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-purple-50 text-purple-600 rounded-lg flex items-center justify-center group-hover:bg-purple-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.manage_quick_items")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("main_menu.manage_quick_desc")}</p>
                        </div>
                    </button>

                    <button
                        onclick={on_manage_inventories_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-orange-300 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-orange-50 text-orange-600 rounded-lg flex items-center justify-center group-hover:bg-orange-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 10h16M4 14h16M4 18h16" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.manage_inventories")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("main_menu.manage_inventories_desc")}</p>
                        </div>
                    </button>

                    <button
                        onclick={on_account_settings_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-purple-300 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-purple-50 text-purple-600 rounded-lg flex items-center justify-center group-hover:bg-purple-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.account_settings")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("main_menu.account_desc")}</p>
                        </div>
                    </button>

                    <button
                        onclick={on_logout_click}
                        class="w-full group relative flex items-center p-4 bg-white border border-gray-200 rounded-xl shadow-sm hover:shadow-md hover:border-gray-400 transition-all duration-200 text-left"
                    >
                        <div class="flex-shrink-0 h-12 w-12 bg-gray-50 text-gray-600 rounded-lg flex items-center justify-center group-hover:bg-gray-600 group-hover:text-white transition-colors duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                            </svg>
                        </div>
                        <div class="ml-4 flex-1">
                            <h3 class="text-lg font-medium text-gray-900">{i18n.t("main_menu.logout")}</h3>
                            <p class="text-sm text-gray-500">{i18n.t("login.sign_out_desc")}</p>
                        </div>
                    </button>
                </div>

                <div class="text-center pt-8">
                    <p class="text-xs text-gray-400">{i18n.t_with("common.copyright", vec![("year", "2026")])}</p>
                </div>
            </div>
        </div>
    }
}

