use std::collections::HashMap;

use dioxus::prelude::*;
use serde::Deserialize;

use crate::{
    components::{footer::Footer, nav::Navbar},
    utils::markdown::parse_markdown,
};

#[derive(Props, PartialEq)]
pub struct DynamicTemplateProps {
    name: String,
    content: String,
    #[props(!optional)]
    template: Option<toml::Value>,
}

pub fn DynamicTemplate(cx: Scope<DynamicTemplateProps>) -> Element {
    let suffix = cx.props.name.split(".").last().unwrap();
    let template = cx.props.template.clone();
    let template = template.unwrap_or_else(|| {
        let mut res = toml::map::Map::new();
        res.insert("using".to_string(), toml::Value::String(String::new()));
        toml::Value::Table(res)
    });
    let template = template.as_table().unwrap();
    let using = template.get("using");
    let mut using = if using.is_none() {
        ""
    } else {
        using.unwrap().as_str().unwrap_or_default()
    };

    cx.render(rsx! {
        div {
            match suffix {
                "md" => {
                    if using.is_empty() {
                        using = "center";
                    }
                    match using {
                        "center" | _ => {
                            rsx! { CenterMarkdown {
                                content: cx.props.content.to_string(),
                                config: template.clone(),
                            } }
                        }
                    }
                },
                "json" => {
                    let content = cx.props.content.to_string();
                    if using.is_empty() {
                        using = "cards";
                    }
                    match using {
                        "cards" | _ => {
                            rsx! {
                                JsonCardList {
                                    content: content,
                                }
                            }
                        }
                    }
                }
                _ => { rsx! { "Content Not Found" } }
            }
        }
    })
}

#[inline_props]
pub fn CenterMarkdown(
    cx: Scope,
    content: String,
    config: toml::map::Map<String, toml::Value>,
) -> Element {
    let html_output = parse_markdown(&content).unwrap();

    let class = if let Some(toml::Value::Table(t)) = config.get("style") {
        generate_prose_class(t.clone())
    } else {
        "prose prose-sm sm:prose-base dark:prose-invert".to_string()
    };

    let hide_navbar = if let Some(toml::Value::Boolean(b)) = config.get("hide-navbar") {
        *b
    } else {
        false
    };

    let hide_footer = if let Some(toml::Value::Boolean(b)) = config.get("hide-footer") {
        *b
    } else {
        false
    };

    cx.render(rsx! {
        section { class: "bg-cover bg-white dark:bg-gray-600",
            if !hide_navbar {
                rsx! { Navbar {} }
            }
            div { class: "flex w-full items-center justify-center container mx-auto px-8",
                div { class: "text-center",
                    div { class: "{class}", dangerous_inner_html: "{html_output}" }
                    if !hide_footer {
                        rsx! { Footer {} }
                    }
                }
            }
        }
    })
}

#[derive(Clone, Deserialize)]
pub struct CardInfo {
    pub title: String,
    pub url: String,
    pub content: String,
    pub footnote: String,
}

#[inline_props]
pub fn JsonCardList(cx: Scope, content: String) -> Element {
    let data = serde_json::from_str::<HashMap<String, Vec<CardInfo>>>(&content);

    if let Err(e) = data {
        return cx.render(rsx! {crate::pages::error::Error { title: "JSON Parse failed".into(), content: format!("{e}") }});
    }
    let data = data.unwrap();

    let displayer = data.iter().map(|(group, value)| {
        rsx! {
            h2 { class: "text-xl font-bold", "# {group}" }
            div { class: "mt-4 grid md:grid-cols-2 gap-2 mb-8",
                value.iter().map(|p| {
                    rsx! {
                        a {
                            class: "block p-4 rounded-lg shadow-lg bg-white w-full sm:w-72 dark:bg-gray-700 hover:bg-gray-200",
                            href: "{p.url}",
                            target: "_blank",
                            h5 {
                                class: "text-gray-900 dark:text-white text-xl leading-tight font-semibold mb-2",
                                "{p.title}"
                            }
                            p {
                                class: "text-gray-700 dark:text-gray-200 text-base mb-2",
                                "{p.content}"
                            }
                            p {
                                class: "text-gray-400 dark:text-gray-500 text-base",
                                "{p.footnote}"
                            }
                        }
                    }
                })
            }
        }
    });

    cx.render(rsx! {
        section { class: "bg-cover bg-white dark:bg-gray-600 dark:text-white",
            Navbar {}
            div { class: "flex h-full w-full items-center justify-center container mx-auto px-8",
                div { class: "max-w-5xl text-center",
                    displayer,
                    Footer {}
                }
            }
        }
    })
}

const AVAILABLE_STYLE_SETTINGS: [&'static str; 26] = [
    "headings",
    "lead",
    "h1",
    "h2",
    "h3",
    "h4",
    "p",
    "a",
    "blockquote",
    "figure",
    "figcaption",
    "strong",
    "em",
    "code",
    "pre",
    "ol",
    "ul",
    "li",
    "table",
    "thead",
    "tr",
    "th",
    "td",
    "img",
    "video",
    "hr",
];

pub fn generate_prose_class(config: toml::map::Map<String, toml::Value>) -> String {
    let mut res = String::from("prose prose-sm sm:prose-base dark:prose-invert");
    for i in AVAILABLE_STYLE_SETTINGS {
        if let Some(toml::Value::String(v)) = config.get(i) {
            let list = v.split(" ").collect::<Vec<&str>>();
            if list.len() >= 1 {
                res.push_str(&format!(" prose-{i}:{}", list.get(0).unwrap()))
            } else {
                res.push_str(&format!("{} ", list.join(&format!(" prose-{i}:"))));
            }
        }
    }
    res
}
