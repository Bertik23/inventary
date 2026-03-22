use crate::api::{reset_password, ResetPasswordRequest};
use crate::i18n::use_i18n;
use crate::router::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(ResetPassword)]
pub fn reset_password_comp(_props: &Props) -> Html {
    let password = use_state(|| String::new());
    let confirm_password = use_state(|| String::new());
    let error = use_state(|| Option::<String>::None);
    let message = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);
    let i18n = use_i18n();

    let location = use_location().expect("Location not found");
    let token = location
        .query::<std::collections::HashMap<String, String>>()
        .ok()
        .and_then(|params| params.get("token").cloned())
        .unwrap_or_default();

    let navigator = use_navigator().unwrap();

    let on_submit = {
        let password = password.clone();
        let confirm_password = confirm_password.clone();
        let error = error.clone();
        let message = message.clone();
        let loading = loading.clone();
        let token = token.clone();
        let navigator = navigator.clone();
        let i18n = i18n.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let password_str = (*password).clone();
            let confirm_str = (*confirm_password).clone();

            if password_str.is_empty() || confirm_str.is_empty() {
                error.set(Some(i18n.t("common.fill_all_fields")));
                return;
            }

            if password_str != confirm_str {
                error.set(Some(i18n.t("login.passwords_dont_match")));
                return;
            }

            if token.is_empty() {
                error.set(Some(i18n.t("login.invalid_reset_token")));
                return;
            }

            loading.set(true);
            error.set(None);

            let req = ResetPasswordRequest {
                token: token.clone(),
                new_password: password_str,
            };

            let error = error.clone();
            let message = message.clone();
            let loading = loading.clone();
            let navigator = navigator.clone();
            let i18n = i18n.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match reset_password(req).await {
                    Ok(_) => {
                        message.set(Some(i18n.t("login.password_updated")));
                        let navigator = navigator.clone();
                        gloo_timers::callback::Timeout::new(3000, move || {
                            navigator.push(&Route::Login);
                        })
                        .forget();
                    }
                    Err(e) => error.set(Some(e)),
                }
                loading.set(false);
            });
        })
    };

    html! {
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-4">
            <div class="w-full max-w-md bg-white rounded-xl shadow-md p-8 border border-gray-100">
                <div class="text-center mb-8">
                    <h1 class="text-2xl font-bold text-gray-900">{i18n.t("login.set_new_password")}</h1>
                    <p class="text-gray-500 mt-2">{i18n.t("login.enter_new_password")}</p>
                </div>

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
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("login.new_password")}</label>
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
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("login.confirm_new_password")}</label>
                        <input
                            type="password"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500 outline-none transition"
                            value={(*confirm_password).clone()}
                            oninput={Callback::from(move |e: InputEvent| {
                                let input: HtmlInputElement = e.target_unchecked_into();
                                confirm_password.set(input.value());
                            })}
                            disabled={*loading}
                        />
                    </div>

                    <button
                        type="submit"
                        class="w-full py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium disabled:opacity-50"
                        disabled={*loading}
                    >
                        {if *loading { i18n.t("login.updating") } else { i18n.t("login.update_password") }}
                    </button>
                </form>
            </div>
        </div>
    }
}
