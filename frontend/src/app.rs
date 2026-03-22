use crate::api::User;
use crate::i18n::I18nProvider;
use crate::router::{switch, Route};
use web_sys::window;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Clone, PartialEq)]
pub struct UserContext {
    pub user: UseStateHandle<Option<User>>,
}

#[derive(Clone, PartialEq)]
pub struct InventoryContext {
    pub inventory_id: UseStateHandle<Option<String>>,
}

#[function_component(App)]
pub fn app() -> Html {
    let user = use_state(|| {
        let storage = window()?.local_storage().ok()??;
        let user_json = storage.get_item("user").ok()??;
        serde_json::from_str::<User>(&user_json).ok()
    });

    let inventory_id = use_state(|| {
        let storage = window()?.local_storage().ok()??;
        storage.get_item("inventory_id").ok()?
    });

    {
        let user = user.clone();
        use_effect_with(user, |user| {
            if let Some(storage) =
                window().and_then(|w| w.local_storage().ok().flatten())
            {
                if let Some(u) = &**user {
                    let json = serde_json::to_string(u).unwrap_or_default();
                    let _ = storage.set_item("user", &json);
                } else {
                    let _ = storage.remove_item("user");
                }
            }
        });
    }

    {
        let inventory_id = inventory_id.clone();
        use_effect_with(inventory_id, |inventory_id| {
            if let Some(storage) =
                window().and_then(|w| w.local_storage().ok().flatten())
            {
                if let Some(id) = &**inventory_id {
                    let _ = storage.set_item("inventory_id", id);
                } else {
                    let _ = storage.remove_item("inventory_id");
                }
            }
        });
    }

    let user_context = UserContext { user };
    let inventory_context = InventoryContext { inventory_id };

    html! {
        <I18nProvider>
            <ContextProvider<UserContext> context={user_context}>
                <ContextProvider<InventoryContext> context={inventory_context}>
                    <BrowserRouter>
                        <Switch<Route> render={switch} />
                    </BrowserRouter>
                </ContextProvider<InventoryContext>>
            </ContextProvider<UserContext>>
        </I18nProvider>
    }
}
