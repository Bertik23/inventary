use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::Route;
use crate::api::{get_custom_item_templates, create_custom_item_template, update_custom_item_template, delete_custom_item_template, CustomItemTemplate, CreateTemplateRequest, UpdateTemplateRequest};
use crate::i18n::use_i18n;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub inventory_id: String,
}

#[function_component(CustomItemManager)]
pub fn custom_item_manager(props: &Props) -> Html {
    let templates = use_state(|| Vec::<CustomItemTemplate>::new());
    let loading = use_state(|| true);
    let error = use_state(|| Option::<String>::None);
    let i18n = use_i18n();
    
    let new_name = use_state(|| String::new());
    let new_unit = use_state(|| "pcs".to_string());
    
    let navigator = use_navigator().unwrap();
    
    let fetch_templates = {
        let templates = templates.clone();
        let loading = loading.clone();
        let error = error.clone();
        let inventory_id = props.inventory_id.clone();
        
        Callback::from(move |_| {
            let templates = templates.clone();
            let loading = loading.clone();
            let error = error.clone();
            let inventory_id = inventory_id.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                match get_custom_item_templates(Some(&inventory_id)).await {
                    Ok(items) => {
                        templates.set(items);
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
    
    {
        let fetch_templates = fetch_templates.clone();
        use_effect_with((), move |_| {
            fetch_templates.emit(());
        });
    }
    
    let on_back = {
        let navigator = navigator.clone();
        Callback::from(move |_| {
            navigator.push(&Route::MainMenu);
        })
    };
    
    let on_add = {
        let fetch_templates = fetch_templates.clone();
        let new_name = new_name.clone();
        let new_unit = new_unit.clone();
        let inventory_id = props.inventory_id.clone();
        
        Callback::from(move |_| {
            let fetch_templates = fetch_templates.clone();
            let name = (*new_name).clone();
            let unit = (*new_unit).clone();
            let inventory_id = inventory_id.clone();
            
            if name.is_empty() { return; }
            
            let new_name_state = new_name.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let req = CreateTemplateRequest {
                    inventory_id: Some(inventory_id),
                    name: name,
                    default_unit: unit,
                };
                
                match create_custom_item_template(req).await {
                    Ok(_) => {
                        fetch_templates.emit(());
                        new_name_state.set(String::new());
                    }
                    Err(e) => log::error!("Failed to create template: {}", e),
                }
            });
        })
    };
    
    let on_unit_change = {
        let fetch_templates = fetch_templates.clone();
        Callback::from(move |(id, unit): (String, String)| {
            let fetch_templates = fetch_templates.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match update_custom_item_template(&id, UpdateTemplateRequest { default_unit: unit }).await {
                    Ok(_) => fetch_templates.emit(()),
                    Err(e) => log::error!("Failed to update template: {}", e),
                }
            });
        })
    };
    
    let on_override = {
        let fetch_templates = fetch_templates.clone();
        let inventory_id = props.inventory_id.clone();
        Callback::from(move |template: CustomItemTemplate| {
            let fetch_templates = fetch_templates.clone();
            let inventory_id = inventory_id.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let req = CreateTemplateRequest {
                    inventory_id: Some(inventory_id),
                    name: template.name,
                    default_unit: template.default_unit,
                };
                match create_custom_item_template(req).await {
                    Ok(_) => fetch_templates.emit(()),
                    Err(e) => log::error!("Failed to override template: {}", e),
                }
            });
        })
    };
    
    let on_delete = {
        let fetch_templates = fetch_templates.clone();
        Callback::from(move |id: String| {
            let fetch_templates = fetch_templates.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match delete_custom_item_template(&id).await {
                    Ok(_) => fetch_templates.emit(()),
                    Err(e) => log::error!("Failed to delete template: {}", e),
                }
            });
        })
    };
    
    html! {
        <div class="max-w-lg mx-auto p-4 min-h-screen bg-gray-50">
            <div class="flex justify-between items-center mb-6">
                <h1 class="text-2xl font-bold text-gray-800">{i18n.t("custom_items.title")}</h1>
                <button class="px-4 py-2 bg-gray-200 text-gray-700 rounded-lg hover:bg-gray-300 transition font-medium" onclick={on_back}>{i18n.t("common.back")}</button>
            </div>
            
            <div class="mb-8 bg-white p-5 rounded-xl shadow-sm border border-gray-100">
                <h2 class="text-lg font-semibold mb-4 text-gray-700">{i18n.t("custom_items.add_new")}</h2>
                <div class="space-y-4">
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("custom_items.name_label")}</label>
                        <input
                            type="text"
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                            value={(*new_name).clone()}
                            oninput={Callback::from({
                                let new_name = new_name.clone();
                                move |e: InputEvent| {
                                    let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                    new_name.set(input.value());
                                }
                            })}
                        />
                    </div>
                    <div>
                        <label class="block text-sm font-medium text-gray-700 mb-1">{i18n.t("custom_items.default_unit")}</label>
                        <select 
                            class="w-full px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 outline-none transition"
                            value={(*new_unit).clone()}
                            onchange={Callback::from({
                                let new_unit = new_unit.clone();
                                move |e: Event| {
                                    let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                    new_unit.set(input.value());
                                }
                            })}
                        >
                            <option value="pcs">{format!("pcs ({})", i18n.t("custom_items.unit_pcs"))}</option>
                            <option value="kg">{format!("kg ({})", i18n.t("custom_items.unit_kg"))}</option>
                            <option value="g">{format!("g ({})", i18n.t("custom_items.unit_g"))}</option>
                            <option value="l">{format!("l ({})", i18n.t("custom_items.unit_l"))}</option>
                            <option value="ml">{format!("ml ({})", i18n.t("custom_items.unit_ml"))}</option>
                        </select>
                    </div>
                    <button 
                        class="w-full py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition font-medium shadow-sm"
                        onclick={on_add}
                        disabled={new_name.is_empty()}
                    >
                        {i18n.t("custom_items.add_button")}
                    </button>
                </div>
            </div>
            
            <h2 class="text-lg font-semibold mb-3 text-gray-700">{i18n.t("custom_items.manage_title")}</h2>
            <p class="text-sm text-gray-500 mb-4">{i18n.t("custom_items.manage_desc")}</p>
            if *loading {
                <div class="flex justify-center p-8">
                    <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                </div>
            } else {
                <div class="space-y-3">
                    {for templates.iter().map(|template| {
                        let id = template.id.clone();
                        let is_global = template.inventory_id.is_none();
                        let current_unit = template.default_unit.clone();
                        let i18n = i18n.clone();
                        
                        html! {
                            <div class="bg-white p-4 rounded-xl shadow-sm border border-gray-100 flex flex-col gap-3">
                                <div class="flex justify-between items-center">
                                    <div>
                                        <h3 class="font-medium text-gray-900">{&template.name}</h3>
                                        if is_global {
                                            <span class="text-[10px] uppercase tracking-wider font-bold text-gray-400">{i18n.t("custom_items.global_default")}</span>
                                        } else {
                                            <span class="text-[10px] uppercase tracking-wider font-bold text-blue-500">{i18n.t("custom_items.inventory_specific")}</span>
                                        }
                                    </div>
                                    <div class="flex items-center gap-2">
                                        if is_global {
                                            <button 
                                                class="px-3 py-1 text-xs bg-blue-50 text-blue-600 rounded-md hover:bg-blue-100 transition font-medium"
                                                onclick={let on_override = on_override.clone(); let t = template.clone(); move |_| on_override.emit(t.clone())}
                                            >
                                                {i18n.t("custom_items.override")}
                                            </button>
                                        } else {
                                            <button 
                                                class="p-2 text-red-400 hover:text-red-600 transition"
                                                onclick={let on_delete = on_delete.clone(); let id = id.clone(); move |_| on_delete.emit(id.clone())}
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                                                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                                                </svg>
                                            </button>
                                        }
                                    </div>
                                </div>
                                <div class="flex items-center gap-2">
                                    <label class="text-xs text-gray-500">{i18n.t("common.unit")}</label>
                                    if is_global {
                                        <span class="text-sm font-medium text-gray-700">{&current_unit}</span>
                                    } else {
                                        <select 
                                            class="text-sm border border-gray-200 rounded px-2 py-1 outline-none focus:ring-1 focus:ring-blue-500"
                                            value={current_unit.clone()}
                                            onchange={let on_unit_change = on_unit_change.clone(); let id = id.clone(); move |e: Event| {
                                                let input: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                                on_unit_change.emit((id.clone(), input.value()));
                                            }}
                                        >
                                            <option value="pcs" selected={current_unit == "pcs"}>{i18n.t("custom_items.unit_pcs")}</option>
                                            <option value="kg" selected={current_unit == "kg"}>{i18n.t("custom_items.unit_kg")}</option>
                                            <option value="g" selected={current_unit == "g"}>{i18n.t("custom_items.unit_g")}</option>
                                            <option value="l" selected={current_unit == "l"}>{i18n.t("custom_items.unit_l")}</option>
                                            <option value="ml" selected={current_unit == "ml"}>{i18n.t("custom_items.unit_ml")}</option>
                                        </select>
                                    }
                                </div>
                            </div>
                        }
                    })}
                </div>
            }
        </div>
    }
}
