use crate::api::{
    forgot_password, get_api_base, login_user, register_user, set_api_base,
    AuthRequest, ForgotPasswordRequest,
};
use crate::app::UserContext;
use crate::i18n::use_i18n;
use crate::router::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(Login)]
pub fn login(_props: &Props) -> Html {
    let username = use_state(|| String::new());
    let email = use_state(|| String::new());
    let password = use_state(|| String::new());
    let api_base = use_state(get_api_base);
    let show_settings = use_state(|| false);
    let error = use_state(|| Option::<String>::None);
    let message = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);
    let is_registering = use_state(|| false);
    let is_forgot_password = use_state(|| false);
    let i18n = use_i18n();

    let user_context =
        use_context::<UserContext>().expect("UserContext not found");
    let navigator = use_navigator().unwrap();

    // Redirect if already logged in
    {
        let user = user_context.user.clone();
        let navigator = navigator.clone();
        use_effect_with(user, move |user| {
            if user.is_some() {
                navigator.push(&Route::Selection);
            }
        });
    }

    let on_submit = {
        let username = username.clone();
        let email = email.clone();
        let password = password.clone();
        let api_base = api_base.clone();
        let error = error.clone();
        let message = message.clone();
        let loading = loading.clone();
        let is_registering = is_registering.clone();
        let is_forgot_password = is_forgot_password.clone();

        let user_context = user_context.clone();
        let navigator = navigator.clone();
        let i18n = i18n.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username_str = (*username).clone();
            let email_str = (*email).clone();
            let password_str = (*password).clone();
            let api_base_str = (*api_base).clone();

            if *is_forgot_password {
                if email_str.is_empty() {
                    error.set(Some(i18n.t("login.enter_email_error")));
                    return;
                }
            } else if *is_registering {
                if username_str.is_empty()
                    || email_str.is_empty()
                    || password_str.is_empty()
                {
                    error.set(Some(i18n.t("common.fill_all_fields")));
                    return;
                }
            } else {
                if username_str.is_empty() || password_str.is_empty() {
                    error.set(Some(i18n.t("common.fill_all_fields")));
                    return;
                }
            }

            // Save API base before continuing
            set_api_base(&api_base_str);

            loading.set(true);
            error.set(None);
            message.set(None);

            let is_reg = *is_registering;
            let is_forgot = *is_forgot_password;
            let error = error.clone();
            let message = message.clone();
            let loading = loading.clone();
            let user_context = user_context.clone();
            let navigator = navigator.clone();
            let i18n = i18n.clone();

            wasm_bindgen_futures::spawn_local(async move {
                if is_forgot {
                    match forgot_password(ForgotPasswordRequest {
                        email: email_str,
                    })
                    .await
                    {
                        Ok(_) => {
                            message.set(Some(i18n.t("login.reset_link_sent")));
                        }
                        Err(e) => error.set(Some(e)),
                    }
                } else {
                    let req = AuthRequest {
                        username: username_str,
                        email: if is_reg { Some(email_str) } else { None },
                        password: password_str,
                    };

                    let result = if is_reg {
                        register_user(req).await
                    } else {
                        login_user(req).await
                    };

                    match result {
                        Ok(user) => {
                            user_context.user.set(Some(user));
                            navigator.push(&Route::Selection);
                        }
                        Err(e) => error.set(Some(e)),
                    }
                }
                loading.set(false);
            });
        })
    };

    html! {
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-4">
            <div class="w-full max-w-md bg-white rounded-xl shadow-md p-8 border border-gray-100">
                <div class="flex justify-between items-start mb-8">
                    <div class="text-left">
                        <h1 class="text-2xl font-bold text-gray-900">
                            {if *is_forgot_password { i18n.t("login.reset_password") } else if *is_registering { i18n.t("login.create_account") } else { i18n.t("login.welcome") }}
                        </h1>
                        <p class="text-gray-500 mt-2">
                            {if *is_forgot_password { i18n.t("login.reset_password_desc") } else if *is_registering { i18n.t("login.sign_up_desc") } else { i18n.t("login.sign_in_desc") }}
                        </p>
                    </div>
                    <button
                        type="button"
                        onclick={
                            let show_settings = show_settings.clone();
                            Callback::from(move |_| show_settings.set(!*show_settings))
                        }
                        class="p-2 text-gray-400 hover:text-gray-600 transition"
                        title="Settings"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                        </svg>
                    </button>
                </div>

                if *show_settings {
                    <div class="mb-6 p-4 bg-blue-50 rounded-lg border border-blue-100 space-y-3">
                        <h3 class="text-sm font-semibold text-blue-800">{i18n.t("login.server_settings")}</h3>
                        <div>
                            <label class="block text-xs font-medium text-blue-700 mb-1">{i18n.t("login.backend_url")}</label>
                            <input
                                type="text"
                                class="w-full px-3 py-2 text-sm border border-blue-200 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                                value={(*api_base).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    api_base.set(input.value());
                                })}
                                placeholder="http://127.0.0.1:8080/api"
                            />
                        </div>
                    </div>
                }

                if let Some(ref err) = *error {
                    <div class="mb-4 p-3 bg-red-100 text-red-700 rounded-lg text-sm">
                        {err}
                    </div>
                }

                if let Some(ref msg) = *message {
                    <div class="mb-4 p-3 bg-green-100 text-green-700 rounded-lg text-sm">
                        {msg}
                    </div>
                }

                <form onsubmit={on_submit} class="space-y-4">
                    if !*is_forgot_password {
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("login.username")}</label>
                            <input
                                type="text"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition"
                                value={(*username).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    username.set(input.value());
                                })}
                                disabled={*loading}
                            />
                        </div>
                    }
                    if *is_forgot_password || *is_registering {
                        <div>
                            <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("login.email")}</label>
                            <input
                                type="email"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition"
                                value={(*email).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    email.set(input.value());
                                })}
                                disabled={*loading}
                            />
                        </div>
                    }
                    if !*is_forgot_password {
                        <div>
                            <div class="flex justify-between items-center mb-1">
                                <label class="block text-sm font-medium text-gray-700">{i18n.t("login.password")}</label>
                                if !*is_registering {
                                    <button
                                        type="button"
                                        class="text-xs text-blue-600 hover:text-blue-800"
                                        onclick={
                                            let is_forgot_password = is_forgot_password.clone();
                                            let error = error.clone();
                                            let message = message.clone();
                                            Callback::from(move |_| {
                                                is_forgot_password.set(true);
                                                error.set(None);
                                                message.set(None);
                                            })
                                        }
                                    >
                                        {i18n.t("login.forgot_password")}
                                    </button>
                                }
                            </div>
                            <input
                                type="password"
                                class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition"
                                value={(*password).clone()}
                                oninput={Callback::from(move |e: InputEvent| {
                                    let input: HtmlInputElement = e.target_unchecked_into();
                                    password.set(input.value());
                                })}
                                disabled={*loading}
                            />
                        </div>
                    }

                    <button
                        type="submit"
                        class="w-full py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium disabled:opacity-50"
                        disabled={*loading}
                    >
                        {if *loading { i18n.t("common.loading") } else if *is_forgot_password { i18n.t("login.send_reset_link") } else if *is_registering { i18n.t("login.sign_up") } else { i18n.t("login.sign_in") }}
                    </button>
                </form>

                <div class="mt-6 text-center space-y-2">
                    if *is_forgot_password {
                        <button
                            class="text-sm text-blue-600 hover:text-blue-800 font-medium block w-full"
                            onclick={
                                let is_forgot_password = is_forgot_password.clone();
                                let error = error.clone();
                                let message = message.clone();
                                Callback::from(move |_| {
                                    is_forgot_password.set(false);
                                    error.set(None);
                                    message.set(None);
                                })
                            }
                            disabled={*loading}
                        >
                            {i18n.t("login.back_to_sign_in")}
                        </button>
                    } else {
                        <button
                            class="text-sm text-blue-600 hover:text-blue-800 font-medium block w-full"
                            onclick={
                                let is_registering = is_registering.clone();
                                let error = error.clone();
                                let message = message.clone();
                                Callback::from(move |_| {
                                    is_registering.set(!*is_registering);
                                    error.set(None);
                                    message.set(None);
                                })
                            }
                            disabled={*loading}
                        >
                            {if *is_registering { i18n.t("login.already_have_account") } else { i18n.t("login.no_account") }}
                        </button>
                    }
                </div>
            </div>

            <div class="mt-8 text-center space-y-1">
                <p class="text-xs text-gray-400">{i18n.t_with("common.copyright", vec![("year", "2026")])}</p>
                <p class="text-[10px] text-gray-300 font-mono">{"v"}{env!("CARGO_PKG_VERSION")}</p>
            </div>
        </div>
    }
}
