use crate::api::{
    create_inventory, get_user_inventories, CreateInventoryRequest, Inventory,
};
use crate::app::{InventoryContext, UserContext};
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
    let new_inv_name = use_state(|| String::new());

    let user_context =
        use_context::<UserContext>().expect("UserContext not found");
    let inventory_context =
        use_context::<InventoryContext>().expect("InventoryContext not found");
    let navigator = use_navigator().unwrap();

    let user_id = match &*user_context.user {
        Some(user) => user.id.clone(),
        None => {
            // Or redirect to login
            return html! { <div>{"Please log in"}</div> };
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
                        let current_id = (*inventory_context.inventory_id).clone();
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
        let show_create = show_create.clone();
        let user_id = user_id.clone();
        let error = error.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();
            let name = (*new_inv_name).clone();
            if name.trim().is_empty() {
                return;
            }

            let inventories = inventories.clone();
            let show_create = show_create.clone();
            let new_inv_name = new_inv_name.clone();
            let user_id = user_id.clone();
            let error = error.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match create_inventory(CreateInventoryRequest {
                    name,
                    owner_id: user_id,
                })
                .await
                {
                    Ok(inv) => {
                        let mut current = (*inventories).clone();
                        current.push(inv);
                        inventories.set(current);
                        show_create.set(false);
                        new_inv_name.set(String::new());
                    }
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    html! {
        <div class="min-h-screen bg-gray-50 p-4">
            <div class="max-w-md mx-auto">
                <div class="flex justify-between items-center mb-6">
                    <h1 class="text-2xl font-bold text-gray-900">{"My Inventories"}</h1>
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
                        title="Logout"
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                        </svg>
                        {"Logout"}
                    </button>
                </div>

                if let Some(ref err) = *error {
                    <div class="mb-4 p-3 bg-red-100 text-red-700 rounded-lg text-sm">{err}</div>
                }

                if *loading {
                    <div class="flex justify-center p-8">
                        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                    </div>
                } else {
                    <div class="space-y-3">
                        {for inventories.iter().map(|inv| {
                            let inventory_context = inventory_context.clone();
                            let navigator = navigator.clone();
                            let id = inv.id.clone();
                            let name = inv.name.clone();

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

                            html! {
                                <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-200 transition flex justify-between items-center">
                                    <span class="font-medium text-gray-800">{&inv.name}</span>
                                    <div class="flex gap-2">
                                        <button onclick={on_share} class="px-3 py-1 bg-gray-200 text-gray-700 rounded-md hover:bg-gray-300">{"Share"}</button>
                                        <button onclick={on_select} class="px-3 py-1 bg-blue-600 text-white rounded-md hover:bg-blue-700">{"Select"}</button>
                                    </div>
                                </div>
                            }
                        })}

                        if *show_create {
                            <form onsubmit={on_create} class="bg-white p-4 rounded-xl shadow-sm border border-gray-200 mt-4">
                                <h3 class="font-medium text-gray-900 mb-3">{"New Inventory"}</h3>
                                <input
                                    type="text"
                                    class="w-full px-3 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none mb-3"
                                    placeholder="Inventory Name"
                                    value={(*new_inv_name).clone()}
                                    oninput={Callback::from(move |e: InputEvent| {
                                        let input: HtmlInputElement = e.target_unchecked_into();
                                        new_inv_name.set(input.value());
                                    })}
                                />
                                <div class="flex gap-2">
                                    <button type="submit" class="flex-1 bg-blue-600 text-white py-2 rounded-lg hover:bg-blue-700 transition">{"Create"}</button>
                                    <button
                                        type="button"
                                        class="flex-1 bg-gray-100 text-gray-700 py-2 rounded-lg hover:bg-gray-200 transition"
                                        onclick={Callback::from(move |_| show_create.set(false))}
                                    >
                                        {"Cancel"}
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
                                {"Create New Inventory"}
                            </button>
                        }
                    </div>
                }
            </div>
        </div>
    }
}

