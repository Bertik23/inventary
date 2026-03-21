use crate::api::{
    list_users, update_user_role, admin_update_user, admin_reset_password, admin_delete_user,
    User, AdminUpdateUserRequest, AdminResetPasswordRequest
};
use crate::app::UserContext;
use crate::router::Route;
use crate::i18n::use_i18n;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq)]
enum AdminAction {
    None,
    Edit(User),
    ResetPassword(User),
    Delete(User),
}

#[function_component(Admin)]
pub fn admin() -> Html {
    let user_context = use_context::<UserContext>().expect("UserContext not found");
    let navigator = use_navigator().unwrap();
    let i18n = use_i18n();

    let current_user = match &*user_context.user {
        Some(u) if u.role == "admin" => u.clone(),
        _ => {
            navigator.push(&Route::MainMenu);
            return html! {};
        }
    };

    let users = use_state(|| Vec::<User>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    let message = use_state(|| Option::<String>::None);
    let action = use_state(|| AdminAction::None);

    // Form states
    let edit_username = use_state(|| String::new());
    let edit_email = use_state(|| String::new());
    let reset_password_val = use_state(|| String::new());

    let fetch_users = {
        let users = users.clone();
        let loading = loading.clone();
        let error = error.clone();
        let admin_id = current_user.id.clone();

        Callback::from(move |_| {
            let users = users.clone();
            let loading = loading.clone();
            let error = error.clone();
            let admin_id = admin_id.clone();

            loading.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match list_users(&admin_id).await {
                    Ok(u) => users.set(u),
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        })
    };

    {
        let fetch_users = fetch_users.clone();
        use_effect_with((), move |_| {
            fetch_users.emit(());
            || ()
        });
    }

    let on_update_role = {
        let admin_id = current_user.id.clone();
        let fetch_users = fetch_users.clone();
        let message = message.clone();
        let error = error.clone();
        let i18n = i18n.clone();

        Callback::from(move |(user_id, new_role): (String, String)| {
            let admin_id = admin_id.clone();
            let fetch_users = fetch_users.clone();
            let message = message.clone();
            let error = error.clone();
            let i18n = i18n.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match update_user_role(&admin_id, &user_id, &new_role).await {
                    Ok(_) => {
                        message.set(Some(i18n.t("admin.role_updated")));
                        fetch_users.emit(());
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let on_submit_edit = {
        let admin_id = current_user.id.clone();
        let action = action.clone();
        let edit_username = edit_username.clone();
        let edit_email = edit_email.clone();
        let fetch_users = fetch_users.clone();
        let message = message.clone();
        let error = error.clone();
        let i18n = i18n.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let AdminAction::Edit(ref user) = *action {
                let admin_id = admin_id.clone();
                let user_id = user.id.clone();
                let username = (*edit_username).clone();
                let email = (*edit_email).clone();
                let action = action.clone();
                let fetch_users = fetch_users.clone();
                let message = message.clone();
                let error = error.clone();
                let i18n = i18n.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let req = AdminUpdateUserRequest {
                        username: Some(username),
                        email: Some(email),
                    };
                    match admin_update_user(&admin_id, &user_id, req).await {
                        Ok(_) => {
                            message.set(Some(i18n.t("admin.profile_updated")));
                            action.set(AdminAction::None);
                            fetch_users.emit(());
                        }
                        Err(e) => error.set(Some(e)),
                    }
                });
            }
        })
    };

    let on_submit_reset = {
        let admin_id = current_user.id.clone();
        let action = action.clone();
        let reset_password_val = reset_password_val.clone();
        let fetch_users = fetch_users.clone();
        let message = message.clone();
        let error = error.clone();
        let i18n = i18n.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            if let AdminAction::ResetPassword(ref user) = *action {
                let admin_id = admin_id.clone();
                let user_id = user.id.clone();
                let new_password = (*reset_password_val).clone();
                let action = action.clone();
                let fetch_users = fetch_users.clone();
                let message = message.clone();
                let error = error.clone();
                let i18n = i18n.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    let req = AdminResetPasswordRequest { new_password };
                    match admin_reset_password(&admin_id, &user_id, req).await {
                        Ok(_) => {
                            message.set(Some(i18n.t("admin.password_reset")));
                            action.set(AdminAction::None);
                            fetch_users.emit(());
                        }
                        Err(e) => error.set(Some(e)),
                    }
                });
            }
        })
    };

    let on_confirm_delete = {
        let admin_id = current_user.id.clone();
        let action = action.clone();
        let fetch_users = fetch_users.clone();
        let message = message.clone();
        let error = error.clone();
        let i18n = i18n.clone();

        Callback::from(move |_| {
            if let AdminAction::Delete(ref user) = *action {
                let admin_id = admin_id.clone();
                let user_id = user.id.clone();
                let action = action.clone();
                let fetch_users = fetch_users.clone();
                let message = message.clone();
                let error = error.clone();
                let i18n = i18n.clone();

                wasm_bindgen_futures::spawn_local(async move {
                    match admin_delete_user(&admin_id, &user_id).await {
                        Ok(_) => {
                            message.set(Some(i18n.t("admin.user_deleted")));
                            action.set(AdminAction::None);
                            fetch_users.emit(());
                        }
                        Err(e) => error.set(Some(e)),
                    }
                });
            }
        })
    };

    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| navigator.push(&Route::MainMenu))
    };

    html! {
        <div class="max-w-6xl mx-auto p-4 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{i18n.t("admin.title")}</h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{i18n.t("common.back")}</button>
            </div>

            if let Some(ref err) = *error {
                <div class="mb-4 p-4 bg-red-100 text-red-800 rounded-xl border border-red-200">{i18n.t_with("common.error", vec![("e", err)])}</div>
            }

            if let Some(ref msg) = *message {
                <div class="mb-4 p-4 bg-green-100 text-green-800 rounded-xl border border-green-200">{msg}</div>
            }

            <div class="bg-white rounded-xl shadow-sm border border-gray-100 overflow-hidden">
                <div class="p-4 border-b border-gray-100 bg-gray-50">
                    <h2 class="font-semibold text-gray-700">{i18n.t("admin.users_list")}</h2>
                </div>
                
                if *loading && users.is_empty() {
                    <div class="p-8 text-center text-gray-500">{i18n.t("common.loading")}</div>
                } else {
                    <div class="overflow-x-auto">
                        <table class="w-full text-left border-collapse">
                            <thead>
                                <tr class="text-xs uppercase text-gray-400 bg-gray-50">
                                    <th class="p-4 font-medium">{i18n.t("login.username")}</th>
                                    <th class="p-4 font-medium">{i18n.t("login.email")}</th>
                                    <th class="p-4 font-medium">{i18n.t("admin.role_label")}</th>
                                    <th class="p-4 font-medium text-right">{i18n.t("admin.actions")}</th>
                                </tr>
                            </thead>
                            <tbody class="divide-y divide-gray-100">
                                {for users.iter().map(|user| {
                                    let user_clone = user.clone();
                                    let is_current = user.id == current_user.id;
                                    
                                    let on_edit = {
                                        let action = action.clone();
                                        let edit_username = edit_username.clone();
                                        let edit_email = edit_email.clone();
                                        let user = user_clone.clone();
                                        Callback::from(move |_| {
                                            edit_username.set(user.username.clone());
                                            edit_email.set(user.email.clone());
                                            action.set(AdminAction::Edit(user.clone()));
                                        })
                                    };

                                    let on_reset = {
                                        let action = action.clone();
                                        let reset_password_val = reset_password_val.clone();
                                        let user = user_clone.clone();
                                        Callback::from(move |_| {
                                            reset_password_val.set(String::new());
                                            action.set(AdminAction::ResetPassword(user.clone()));
                                        })
                                    };

                                    let on_delete = {
                                        let action = action.clone();
                                        let user = user_clone.clone();
                                        Callback::from(move |_| action.set(AdminAction::Delete(user.clone())))
                                    };

                                    let on_toggle_role = {
                                        let on_update_role = on_update_role.clone();
                                        let user_id = user.id.clone();
                                        let next_role = if user.role == "admin" { "user" } else { "admin" };
                                        Callback::from(move |_| on_update_role.emit((user_id.clone(), next_role.to_string())))
                                    };

                                    html! {
                                        <tr class="hover:bg-gray-50 transition-colors">
                                            <td class="p-4 text-gray-900 font-medium">{&user.username}</td>
                                            <td class="p-4 text-gray-600 text-sm">{&user.email}</td>
                                            <td class="p-4">
                                                <button 
                                                    onclick={on_toggle_role}
                                                    disabled={is_current}
                                                    class={classes!(
                                                        "px-2", "py-1", "rounded-full", "text-xs", "font-medium", "transition-colors",
                                                        if user.role == "admin" { "bg-purple-100 text-purple-700 hover:bg-purple-200" } else { "bg-blue-100 text-blue-700 hover:bg-blue-200" }
                                                    )}
                                                >
                                                    {if user.role == "admin" { i18n.t("admin.admin_role") } else { i18n.t("admin.user_role") }}
                                                </button>
                                            </td>
                                            <td class="p-4 text-right space-x-2">
                                                if !is_current {
                                                    <button onclick={on_edit} class="text-xs font-medium px-2 py-1 bg-gray-100 text-gray-600 rounded hover:bg-gray-200 transition">
                                                        {i18n.t("admin.edit_user")}
                                                    </button>
                                                    <button onclick={on_reset} class="text-xs font-medium px-2 py-1 bg-yellow-50 text-yellow-700 rounded border border-yellow-100 hover:bg-yellow-100 transition">
                                                        {i18n.t("admin.reset_password")}
                                                    </button>
                                                    <button onclick={on_delete} class="text-xs font-medium px-2 py-1 bg-red-50 text-red-700 rounded border border-red-100 hover:bg-red-100 transition">
                                                        {i18n.t("admin.delete_user")}
                                                    </button>
                                                }
                                            </td>
                                        </tr>
                                    }
                                })}
                            </tbody>
                        </table>
                    </div>
                }
            </div>

            // Modals
            {match &*action {
                AdminAction::None => html! {},
                AdminAction::Edit(_) => html! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                        <div class="bg-white rounded-xl shadow-xl max-w-md w-full p-6">
                            <h3 class="text-xl font-bold mb-4">{i18n.t("admin.edit_user")}</h3>
                            <form onsubmit={on_submit_edit} class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("admin.new_username")}</label>
                                    <input 
                                        type="text" 
                                        class="w-full px-4 py-2 border rounded-lg outline-none focus:ring-2 focus:ring-blue-500"
                                        value={(*edit_username).clone()}
                                        oninput={let edit_username = edit_username.clone(); Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            edit_username.set(input.value());
                                        })}
                                    />
                                </div>
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("admin.new_email")}</label>
                                    <input 
                                        type="email" 
                                        class="w-full px-4 py-2 border rounded-lg outline-none focus:ring-2 focus:ring-blue-500"
                                        value={(*edit_email).clone()}
                                        oninput={let edit_email = edit_email.clone(); Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            edit_email.set(input.value());
                                        })}
                                    />
                                </div>
                                <div class="flex gap-2 pt-2">
                                    <button type="submit" class="flex-1 py-2 bg-blue-600 text-white rounded-lg font-medium hover:bg-blue-700 transition">
                                        {i18n.t("common.save")}
                                    </button>
                                    <button type="button" onclick={let action = action.clone(); move |_| action.set(AdminAction::None)} class="flex-1 py-2 bg-gray-200 text-gray-700 rounded-lg font-medium hover:bg-gray-300 transition">
                                        {i18n.t("common.cancel")}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                },
                AdminAction::ResetPassword(_) => html! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                        <div class="bg-white rounded-xl shadow-xl max-w-md w-full p-6">
                            <h3 class="text-xl font-bold mb-4">{i18n.t("admin.reset_password")}</h3>
                            <form onsubmit={on_submit_reset} class="space-y-4">
                                <div>
                                    <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("admin.new_password")}</label>
                                    <input 
                                        type="password" 
                                        class="w-full px-4 py-2 border rounded-lg outline-none focus:ring-2 focus:ring-blue-500"
                                        value={(*reset_password_val).clone()}
                                        oninput={let reset_password_val = reset_password_val.clone(); Callback::from(move |e: InputEvent| {
                                            let input: HtmlInputElement = e.target_unchecked_into();
                                            reset_password_val.set(input.value());
                                        })}
                                    />
                                </div>
                                <div class="flex gap-2 pt-2">
                                    <button type="submit" class="flex-1 py-2 bg-yellow-600 text-white rounded-lg font-medium hover:bg-yellow-700 transition">
                                        {i18n.t("admin.reset_password")}
                                    </button>
                                    <button type="button" onclick={let action = action.clone(); move |_| action.set(AdminAction::None)} class="flex-1 py-2 bg-gray-200 text-gray-700 rounded-lg font-medium hover:bg-gray-300 transition">
                                        {i18n.t("common.cancel")}
                                    </button>
                                </div>
                            </form>
                        </div>
                    </div>
                },
                AdminAction::Delete(user) => html! {
                    <div class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                        <div class="bg-white rounded-xl shadow-xl max-w-md w-full p-6 text-center">
                            <div class="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-100 mb-4">
                                <svg class="h-6 w-6 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                </svg>
                            </div>
                            <h3 class="text-xl font-bold mb-2">{i18n.t("admin.delete_user")}</h3>
                            <p class="text-sm text-gray-500 mb-6">{i18n.t_with("admin.delete_confirm_desc", vec![("name", &user.username)])}</p>
                            <div class="flex gap-2">
                                <button onclick={on_confirm_delete} class="flex-1 py-2 bg-red-600 text-white rounded-lg font-medium hover:bg-red-700 transition">
                                    {i18n.t("common.delete")}
                                </button>
                                <button onclick={let action = action.clone(); move |_| action.set(AdminAction::None)} class="flex-1 py-2 bg-gray-200 text-gray-700 rounded-lg font-medium hover:bg-gray-300 transition">
                                    {i18n.t("common.cancel")}
                                </button>
                            </div>
                        </div>
                    </div>
                },
            }}
        </div>
    }
}
