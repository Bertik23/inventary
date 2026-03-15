use crate::api::{login_user, register_user, AuthRequest, get_api_base, set_api_base};
use crate::app::UserContext;
use crate::router::Route;
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {}

#[function_component(Login)]
pub fn login(_props: &Props) -> Html {
    let username = use_state(|| String::new());
    let password = use_state(|| String::new());
    let api_base = use_state(get_api_base);
    let show_settings = use_state(|| false);
    let error = use_state(|| Option::<String>::None);
    let loading = use_state(|| false);
    let is_registering = use_state(|| false);

    let user_context =
        use_context::<UserContext>().expect("UserContext not found");
    let navigator = use_navigator().unwrap();

    let on_submit = {
        let username = username.clone();
        let password = password.clone();
        let api_base = api_base.clone();
        let error = error.clone();
        let loading = loading.clone();
        let is_registering = is_registering.clone();

        let user_context = user_context.clone();
        let navigator = navigator.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let username_str = (*username).clone();
            let password_str = (*password).clone();
            let api_base_str = (*api_base).clone();

            if username_str.is_empty() || password_str.is_empty() {
                error.set(Some("Please fill in all fields".to_string()));
                return;
            }

            // Save API base before continuing
            set_api_base(&api_base_str);

            loading.set(true);
            error.set(None);

            let req = AuthRequest {
                username: username_str,
                password: password_str,
            };

            let is_reg = *is_registering;
            let error = error.clone();
            let loading = loading.clone();
            let user_context = user_context.clone();
            let navigator = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let result = if is_reg {
                    register_user(req).await
                } else {
                    login_user(req).await
                };

                loading.set(false);

                match result {
                    Ok(user) => {
                        user_context.user.set(Some(user));
                        navigator.push(&Route::Selection);
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    html! {
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-4">
            <div class="w-full max-w-md bg-white rounded-xl shadow-md p-8 border border-gray-100">
                <div class="flex justify-between items-start mb-8">
                    <div class="text-left">
                        <h1 class="text-2xl font-bold text-gray-900">
                            {if *is_registering { "Create Account" } else { "Welcome Back" }}
                        </h1>
                        <p class="text-gray-500 mt-2">
                            {if *is_registering { "Sign up to manage your inventory" } else { "Sign in to your account" }}
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
                        <h3 class="text-sm font-semibold text-blue-800">{"Server Settings"}</h3>
                        <div>
                            <label class="block text-xs font-medium text-blue-700 mb-1">{"Backend API URL"}</label>
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

                <form onsubmit={on_submit} class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{"Username"}</label>
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
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{"Password"}</label>
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

                    <button
                        type="submit"
                        class="w-full py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium disabled:opacity-50"
                        disabled={*loading}
                    >
                        {if *loading { "Please wait..." } else if *is_registering { "Sign Up" } else { "Sign In" }}
                    </button>
                </form>

                <div class="mt-6 text-center">
                    <button
                        class="text-sm text-blue-600 hover:text-blue-800 font-medium"
                        onclick={
                            let is_registering = is_registering.clone();
                            Callback::from(move |_| is_registering.set(!*is_registering))
                        }
                        disabled={*loading}
                    >
                        {if *is_registering { "Already have an account? Sign in" } else { "Don't have an account? Sign up" }}
                    </button>
                </div>
            </div>
        </div>
    }
}

