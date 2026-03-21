use yew::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use gloo_storage::{Storage, LocalStorage};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Language {
    En,
    Cs,
}

impl Language {
    pub fn to_str(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Cs => "cs",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "cs" => Language::Cs,
            _ => Language::En,
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct I18nContext {
    pub language: UseStateHandle<Language>,
    pub translations: HashMap<String, String>,
}

impl I18nContext {
    pub fn t(&self, key: &str) -> String {
        self.translations.get(key).cloned().unwrap_or_else(|| key.to_string())
    }

    pub fn t_with(&self, key: &str, params: Vec<(&str, &str)>) -> String {
        let mut s = self.t(key);
        for (k, v) in params {
            s = s.replace(&format!("{{{}}}", k), v);
        }
        s
    }
}

fn load_translations(lang: Language) -> HashMap<String, String> {
    let json_str = match lang {
        Language::En => include_str!("../locales/en.json"),
        Language::Cs => include_str!("../locales/cs.json"),
    };

    let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
    let mut map = HashMap::new();
    
    if let Some(obj) = v.as_object() {
        for (section_name, section_val) in obj {
            if let Some(section_obj) = section_val.as_object() {
                for (key, val) in section_obj {
                    if let Some(s) = val.as_str() {
                        map.insert(format!("{}.{}", section_name, key), s.to_string());
                    }
                }
            }
        }
    }
    
    map
}

#[derive(Properties, PartialEq)]
pub struct I18nProviderProps {
    #[prop_or_default]
    pub children: Children,
}

#[function_component(I18nProvider)]
pub fn i18n_provider(props: &I18nProviderProps) -> Html {
    let initial_lang = LocalStorage::get::<String>("lang")
        .map(|s| Language::from_str(&s))
        .unwrap_or(Language::En);
        
    let language = use_state(|| initial_lang);
    let translations = use_memo(*language, |lang| load_translations(*lang));

    {
        let language = language.clone();
        use_effect_with(*language, move |lang| {
            let _ = LocalStorage::set("lang", lang.to_str());
        });
    }

    let context = I18nContext {
        language,
        translations: (*translations).clone(),
    };

    html! {
        <ContextProvider<I18nContext> context={context}>
            {props.children.clone()}
        </ContextProvider<I18nContext>>
    }
}

#[hook]
pub fn use_i18n() -> I18nContext {
    use_context::<I18nContext>().expect("I18nContext not found")
}
