use crate::api::{
    get_inventory_users, share_inventory, unshare_inventory,
    ShareInventoryRequest, SharedUser, UnshareInventoryRequest,
};
use crate::router::Route;
use crate::i18n::use_i18n;
use log;
use web_sys::{HtmlInputElement, HtmlOptionElement};
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub inventory_id: String,
}

#[function_component(ShareInventory)]
pub fn share_inventory(props: &Props) -> Html {
    let shared_users = use_state(|| Vec::<SharedUser>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    let new_username = use_state(|| String::new());
    let new_role = use_state(|| "viewer".to_string());
    let i18n = use_i18n();

    let inventory_id = props.inventory_id.clone();
    let navigator = use_navigator().unwrap();

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::Selection);
        })
    };

    let fetch_users = {
        let shared_users = shared_users.clone();
        let loading = loading.clone();
        let error = error.clone();
        let inventory_id = inventory_id.clone();
        Callback::from(move |()| {
            let shared_users = shared_users.clone();
            let loading = loading.clone();
            let error = error.clone();
            let inventory_id = inventory_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match get_inventory_users(&inventory_id).await {
                    Ok(users) => {
                        shared_users.set(users);
                        loading.set(false);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    use_effect_with((), {
        let fetch_users = fetch_users.clone();
        move |_| {
            fetch_users.emit(());
        }
    });

    let on_share = {
        let new_username = new_username.clone();
        let new_role = new_role.clone();
        let inventory_id = inventory_id.clone();
        let error = error.clone();
        let fetch_users = fetch_users.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username = (*new_username).clone();
            let role = (*new_role).clone();
            if username.is_empty() {
                error.set(Some(i18n.t("share.username_empty")));
                return;
            }

            let inventory_id = inventory_id.clone();
            let error = error.clone();
            let fetch_users = fetch_users.clone();
            let new_username = new_username.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let req = ShareInventoryRequest { username, role };
                match crate::api::share_inventory(&inventory_id, req).await {
                    Ok(_) => {
                        log::info!("Inventory shared successfully");
                        new_username.set(String::new());
                        fetch_users.emit(());
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
            });
        })
    };

    let on_unshare = {
        let inventory_id = inventory_id.clone();
        let error = error.clone();
        let fetch_users = fetch_users.clone();
        Callback::from(move |user_id: String| {
            let inventory_id = inventory_id.clone();
            let error = error.clone();
            let fetch_users = fetch_users.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let req = UnshareInventoryRequest { user_id };
                match unshare_inventory(&inventory_id, req).await {
                    Ok(_) => {
                        log::info!("User removed from inventory");
                        fetch_users.emit(());
                    }
                    Err(e) => {
                        error.set(Some(e));
                    }
                }
            });
        })
    };

    html! {
        <div class="max-w-lg mx-auto p-4 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{i18n.t("share.title")}</h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{i18n.t("common.back")}</button>
            </div>

            if let Some(ref err) = *error {
                <div class="mb-4 p-3 bg-red-100 text-red-700 rounded-lg text-sm">{i18n.t_with("common.error", vec![("e", err)])}</div>
            }

            <div class="bg-white p-5 rounded-xl shadow-sm border border-gray-100 mb-6">
                <h2 class="text-lg font-semibold mb-3 text-gray-700">{i18n.t("share.add_user")}</h2>
                <form onsubmit={on_share} class="flex gap-2">
                    <input
                        type="text"
                        class="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition bg-gray-50"
                        placeholder={i18n.t("share.username_placeholder")}
                        value={(*new_username).clone()}
                        oninput={Callback::from({
                            let new_username = new_username.clone();
                            move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                new_username.set(input.value());
                            }
                        })}
                    />
                    <select class="px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition bg-gray-50"
                        onchange={Callback::from({
                            let new_role = new_role.clone();
                            move |e: Event| {
                                let select: HtmlOptionElement = e.target_unchecked_into();
                                new_role.set(select.value());
                            }
                        })}>
                        <option value="viewer">{i18n.t("share.role_viewer")}</option>
                        <option value="editor">{i18n.t("share.role_editor")}</option>
                    </select>
                    <button type="submit" class="px-6 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 transition font-medium shadow-sm">
                        {i18n.t("share.add_button")}
                    </button>
                </form>
            </div>

            <h2 class="text-lg font-semibold mb-3 text-gray-700">{i18n.t("share.shared_users")}</h2>
            if *loading {
                <div class="flex justify-center p-8">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                </div>
            } else {
                <div class="space-y-3">
                    {for shared_users.iter().map(|user| {
                        let user_id = user.id.clone();
                        let on_unshare = on_unshare.clone();
                        html! {
                            <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-100 flex justify-between items-center">
                                <div>
                                    <p class="font-semibold text-gray-900">{&user.username}</p>
                                    <p class="text-sm text-gray-500">{&user.role}</p>
                                </div>
                                <button onclick={Callback::from(move |_| on_unshare.emit(user_id.clone()))} class="px-3 py-1 bg-red-100 text-red-700 rounded-md hover:bg-red-200">{i18n.t("common.delete")}</button>
                            </div>
                        }
                    })}
                </div>
            }
        </div>
    }
}
