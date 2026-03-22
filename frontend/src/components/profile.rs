use crate::api::{
    change_password, delete_user, update_user, ChangePasswordRequest,
    UpdateUserRequest,
};
use crate::app::{InventoryContext, UserContext};
use crate::i18n::use_i18n;
use crate::router::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[function_component(Profile)]
pub fn profile() -> Html {
    let user_context =
        use_context::<UserContext>().expect("UserContext not found");
    let inventory_context =
        use_context::<InventoryContext>().expect("InventoryContext not found");
    let navigator = use_navigator().unwrap();
    let i18n = use_i18n();

    let user = match &*user_context.user {
        Some(u) => u.clone(),
        None => {
            navigator.push(&Route::Login);
            return html! {};
        }
    };

    let username = use_state(|| user.username.clone());
    let email = use_state(|| user.email.clone());
    let current_password = use_state(|| String::new());
    let new_password = use_state(|| String::new());
    let confirm_password = use_state(|| String::new());

    let loading = use_state(|| false);
    let error = use_state(|| Option::<String>::None);
    let message = use_state(|| Option::<String>::None);
    let show_delete_confirm = use_state(|| false);

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::MainMenu);
        })
    };

    let on_update_profile = {
        let username = username.clone();
        let email = email.clone();
        let user_id = user.id.clone();
        let user_context = user_context.clone();
        let loading = loading.clone();
        let error = error.clone();
        let message = message.clone();
        let i18n = i18n.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let user_id = user_id.clone();
            let new_username = (*username).clone();
            let new_email = (*email).clone();

            let loading = loading.clone();
            let error = error.clone();
            let message = message.clone();
            let user_context = user_context.clone();
            let i18n = i18n.clone();

            loading.set(true);
            error.set(None);
            message.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let req = UpdateUserRequest {
                    username: Some(new_username),
                    email: Some(new_email),
                };
                match update_user(&user_id, req).await {
                    Ok(updated_user) => {
                        user_context.user.set(Some(updated_user));
                        message.set(Some(i18n.t("account.profile_updated")));
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        })
    };

    let on_change_password = {
        let current_p = current_password.clone();
        let new_p = new_password.clone();
        let confirm_p = confirm_password.clone();
        let user_id = user.id.clone();
        let loading = loading.clone();
        let error = error.clone();
        let message = message.clone();
        let i18n = i18n.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let cur = (*current_p).clone();
            let new = (*new_p).clone();
            let conf = (*confirm_p).clone();

            if cur.is_empty() || new.is_empty() {
                error.set(Some(i18n.t("common.fill_all_fields")));
                return;
            }

            if new != conf {
                error.set(Some(i18n.t("login.passwords_dont_match")));
                return;
            }

            let user_id = user_id.clone();
            let loading = loading.clone();
            let error = error.clone();
            let message = message.clone();
            let i18n = i18n.clone();
            let current_p = current_p.clone();
            let new_p = new_p.clone();
            let confirm_p = confirm_p.clone();

            loading.set(true);
            error.set(None);
            message.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let req = ChangePasswordRequest {
                    current_password: cur,
                    new_password: new,
                };
                match change_password(&user_id, req).await {
                    Ok(_) => {
                        message.set(Some(i18n.t("account.password_changed")));
                        current_p.set(String::new());
                        new_p.set(String::new());
                        confirm_p.set(String::new());
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        })
    };

    let on_delete_account = {
        let user_id = user.id.clone();
        let user_context = user_context.clone();
        let inventory_context = inventory_context.clone();
        let navigator = navigator.clone();
        let loading = loading.clone();
        let error = error.clone();
        let i18n = i18n.clone();

        Callback::from(move |_| {
            let user_id = user_id.clone();
            let user_context = user_context.clone();
            let inventory_context = inventory_context.clone();
            let navigator = navigator.clone();
            let loading = loading.clone();
            let error = error.clone();

            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match delete_user(&user_id).await {
                    Ok(_) => {
                        user_context.user.set(None);
                        inventory_context.inventory_id.set(None);
                        navigator.push(&Route::Login);
                    }
                    Err(e) => {
                        error.set(Some(e));
                        loading.set(false);
                    }
                }
            });
        })
    };

    html! {
        <div class="max-w-lg mx-auto p-4 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{i18n.t("account.title")}</h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{i18n.t("common.back")}</button>
            </div>

            if let Some(ref err) = *error {
                <div class="mb-4 p-4 bg-red-100 text-red-800 rounded-xl border border-red-200">{i18n.t_with("common.error", vec![("e", err)])}</div>
            }

            if let Some(ref msg) = *message {
                <div class="mb-4 p-4 bg-green-100 text-green-800 rounded-xl border border-green-200">{msg}</div>
            }

            // Profile Section
            <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100 mb-6">
                <h2 class="text-lg font-semibold mb-4 text-gray-700">{i18n.t("account.profile_section")}</h2>
                <form onsubmit={on_update_profile} class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("login.username")}</label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                            value={(*username).clone()}
                            oninput={Callback::from({
                                let username = username.clone();
                                move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    username.set(input.value());
                                }
                            })}
                            disabled={*loading}
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("login.email")}</label>
                        <input
                            type="email"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                            value={(*email).clone()}
                            oninput={Callback::from({
                                let email = email.clone();
                                move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    email.set(input.value());
                                }
                            })}
                            disabled={*loading}
                        />
                    </div>
                    <button
                        type="submit"
                        class="w-full py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium shadow-sm disabled:opacity-50"
                        disabled={*loading}
                    >
                        {i18n.t("account.update_profile")}
                    </button>
                </form>
            </div>

            // Password Section
            <div class="bg-white p-6 rounded-xl shadow-sm border border-gray-100 mb-6">
                <h2 class="text-lg font-semibold mb-4 text-gray-700">{i18n.t("account.security_section")}</h2>
                <form onsubmit={on_change_password} class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("account.current_password")}</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                            value={(*current_password).clone()}
                            oninput={Callback::from({
                                let current_password = current_password.clone();
                                move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    current_password.set(input.value());
                                }
                            })}
                            disabled={*loading}
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("account.new_password")}</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                            value={(*new_password).clone()}
                            oninput={Callback::from({
                                let new_password = new_password.clone();
                                move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    new_password.set(input.value());
                                }
                            })}
                            disabled={*loading}
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("account.confirm_new_password")}</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                            value={(*confirm_password).clone()}
                            oninput={Callback::from({
                                let confirm_password = confirm_password.clone();
                                move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    confirm_password.set(input.value());
                                }
                            })}
                            disabled={*loading}
                        />
                    </div>
                    <button
                        type="submit"
                        class="w-full py-2 bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition font-medium shadow-sm disabled:opacity-50"
                        disabled={*loading}
                    >
                        {i18n.t("account.change_password")}
                    </button>
                </form>
            </div>

            // Danger Zone
            <div class="bg-red-50 p-6 rounded-xl shadow-sm border border-red-100 mb-6">
                <h2 class="text-lg font-semibold mb-2 text-red-800">{i18n.t("account.danger_section")}</h2>
                <p class="text-sm text-red-600 mb-4">{i18n.t("account.delete_confirm_desc")}</p>

                if !*show_delete_confirm {
                    <button
                        onclick={let show_delete_confirm = show_delete_confirm.clone(); move |_| show_delete_confirm.set(true)}
                        class="w-full py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition font-medium shadow-sm"
                    >
                        {i18n.t("account.delete_account")}
                    </button>
                } else {
                    <div class="space-y-3">
                        <p class="font-bold text-red-800 text-center">{i18n.t("account.delete_confirm_title")}</p>
                        <div class="flex gap-2">
                            <button
                                onclick={on_delete_account}
                                class="flex-1 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition font-medium shadow-sm"
                                disabled={*loading}
                            >
                                {i18n.t("account.delete_button")}
                            </button>
                            <button
                                onclick={let show_delete_confirm = show_delete_confirm.clone(); move |_| show_delete_confirm.set(false)}
                                class="flex-1 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium"
                                disabled={*loading}
                            >
                                {i18n.t("common.cancel")}
                            </button>
                        </div>
                    </div>
                }
            </div>
        </div>
    }
}
