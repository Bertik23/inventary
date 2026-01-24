use yew::prelude::*;
use yew_router::prelude::*;
use crate::router::{switch, Route};
use crate::api::User;

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
    let user = use_state(|| Option::<User>::None);
    let inventory_id = use_state(|| Option::<String>::None);

    let user_context = UserContext { user };
    let inventory_context = InventoryContext { inventory_id };

    html! {
        <ContextProvider<UserContext> context={user_context}>
            <ContextProvider<InventoryContext> context={inventory_context}>
                <BrowserRouter>
                    <Switch<Route> render={switch} />
                </BrowserRouter>
            </ContextProvider<InventoryContext>>
        </ContextProvider<UserContext>>
    }
}
