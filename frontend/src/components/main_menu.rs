use yew::prelude::*;
use crate::app::AppPage;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub navigate: Callback<AppPage>,
}

#[function_component(MainMenu)]
pub fn main_menu(props: &Props) -> Html {
    let navigate = props.navigate.clone();
    
    let on_add_click = {
        let navigate = navigate.clone();
        Callback::from(move |_| {
            navigate.emit(AppPage::AddItem);
        })
    };
    
    let on_remove_click = {
        let navigate = navigate.clone();
        Callback::from(move |_| {
            navigate.emit(AppPage::RemoveItem);
        })
    };
    
    let on_show_inventory_click = {
        let navigate = navigate.clone();
        Callback::from(move |_| {
            navigate.emit(AppPage::InventoryList);
        })
    };
    
    html! {
        <div class="min-h-screen bg-gray-50 flex flex-col items-center justify-center p-4">
            <div class="w-full max-w-md space-y-8">
                <div class="text-center">
                    <div class="mx-auto h-20 w-20 bg-blue-600 rounded-2xl flex items-center justify-center shadow-lg mb-6 transform -rotate-3">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-10 w-10 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                        </svg>
                    </div>
                    <h1 class="text-3xl font-extrabold text-gray-900 tracking-tight">{"Inventary"}</h1>
                    <p class="mt-2 text-gray-500">{"Manage your stock with ease"}</p>
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
                            <h3 class="text-lg font-medium text-gray-900">{"Add Item"}</h3>
                            <p class="text-sm text-gray-500">{"Scan barcode or search manually"}</p>
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
                            <h3 class="text-lg font-medium text-gray-900">{"Remove Item"}</h3>
                            <p class="text-sm text-gray-500">{"Decrease quantity or delete"}</p>
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
                            <h3 class="text-lg font-medium text-gray-900">{"Show Inventory"}</h3>
                            <p class="text-sm text-gray-500">{"View full list of items"}</p>
                        </div>
                    </button>
                </div>
                
                <div class="text-center pt-8">
                    <p class="text-xs text-gray-400">{"© 2024 Inventary App"}</p>
                </div>
            </div>
        </div>
    }
}
