#![allow(non_snake_case)]

use dioxus::prelude::*;
use dioxus::desktop::{use_asset_handler, wry::http::Response};
use tracing::Level;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entry {
    Phrase(String),
    Drawing(Vec<u8>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PictionaryEntry {
    pub author: String,
    pub entry: Entry,
}

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    // Init logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    let window = dioxus::desktop::WindowBuilder::new()
        .with_inner_size(dioxus::desktop::LogicalSize::new(800.0, 800.0))
        .with_title("Telephone Pictionary")
        .with_resizable(true);

    let config = dioxus::desktop::Config::new()
        .with_window(window);
    LaunchBuilder::new().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        Router::<Route> {}
    }
}

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[component]
fn Home() -> Element {
    let state = use_signal(|| Vec::<Entry>::new());
    let view_all = use_signal(|| false);

    use_asset_handler("entry", move |request, response| {
        match request.uri().path().strip_prefix("/entry/") {
            Some(s) => {
                match state()[s.parse::<usize>().unwrap()].clone() {
                    Entry::Drawing(v) => response.respond(Response::new(v)),
                    Entry::Phrase(_s) => return,
                }
            },
            None => return,
        }
    });

    rsx! {
        if !view_all() {
            EntryDisplay { state }
        }
        ViewAllBtn { view_all }
        GameControls { state }
        if view_all() {
            GameReview { state }
        }
    }
}

#[component]
fn GameReview(state: Signal<Vec<Entry>>) -> Element {
    rsx! { 
        {state().iter().enumerate().map(|(index, e)| {
            match e {
                Entry::Phrase(phrase) => rsx! { PhraseDisplay { phrase } },
                Entry::Drawing(_v) => rsx! { DrawingDisplay { index } },
            }
        })}
    }
}

#[component]
fn ViewAllBtn(view_all: Signal<bool>) -> Element {
    rsx! {
        p {
            button {
                width: "80px",
                onclick: move |_evt| {
                    *view_all.write() = !view_all();
                },
                if view_all() {
                    "Go Back"
                }
                else {
                    "End Game"
                }
            }
        }
    }
}

#[component]
fn PhraseDisplay(phrase: String) -> Element {
    rsx! {
        div {
            border: "1px solid black",
            {phrase.lines().map(|line| rsx!{ "{line}" br{} })},
        }
    }
}

#[component]
fn DrawingDisplay(index: usize) -> Element {
    rsx! {
        img {
            width:"600px",
            height:"600px",
            "object-fit": "contain",
            src: "entry/{index}"
        }
    }    
}

#[component]
fn EntryDisplay(state: Signal<Vec<Entry>>) -> Element {
    match state().last() {
        Some(last) => match last {
            Entry::Phrase(phrase) => rsx! { 
                div {
                    p { 
                        "Draw: "
                        PhraseDisplay { phrase }
                        "then upload the image" 
                    }
                    ImgSelector { state } 
                }
            },
            Entry::Drawing(_v) => rsx! { 
                div {
                    DrawingDisplay { 
                        index: state().len() - 1 
                    }
                    div { "What is this?" }
                    PhraseInput { state }
                }
            },
        }
        None => rsx! {
            div {
                div { "Write something!" }
                PhraseInput { state }
            }
        }
    }
}

#[component]
fn ImgSelector(state: Signal<Vec<Entry>>) -> Element {
    rsx! {
        input {
            name: "picture",
            r#type: "file",
            accept: ".png,.jpg",
            onchange: move |evt| {
                async move {
                    if let Some(file_engine) = &evt.files() {
                        let mut temp = state();
                        if file_engine.files().len() > 0 {
                            temp.push(Entry::Drawing(file_engine.read_file(
                                file_engine.files()[0].as_str())
                                    .await.unwrap()));
                            *state.write() = temp;
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PhraseInput(state: Signal<Vec<Entry>>) -> Element {
    let mut small_state = use_signal(|| String::from(""));
    rsx! {
        div {
            textarea {
                cols: 80,
                rows: 3,
                name: "caption",
                wrap: "hard",
                oninput: move |event| *small_state.write() = event.value(),
                "{ small_state() }"
            }
        }
        button {
            width: "80px",
            onclick: move |_evt| {
                let mut temp = state();
                temp.push(Entry::Phrase(small_state()));
                *state.write() = temp;
            },
            "OK"
        }
    }
}

#[component]
fn NewBtn(state: Signal<Vec<Entry>>) -> Element {
    rsx! {
        p {
            button {
                width: "80px",
                ondoubleclick: move |_evt| {
                    let mut temp = state();
                    temp.clear();
                    *state.write() = temp;
                },
                "New"
            }
        }
    }
}

#[component]
fn SaveBtn(state: Signal<Vec<Entry>>) -> Element {
    let mut small_state = use_signal(|| String::from(""));
    rsx! {
        p {
            button {
                disabled: small_state().is_empty(),
                width: "80px",
                onclick: move |_evt| {
                    async move {
                        let mut filename = small_state();
                        if !filename.ends_with(".tpi") {
                            filename.push_str(".tpi");
                        }
                        let mut file = File::create(filename).unwrap();
                        file.write_all(
                            serde_json::to_vec(&state()).unwrap().as_slice()
                        ).unwrap();
                    }
                }, 
                "Save"
            }
            "File: "
            input {
                name: "save",
                r#type: "text",
                accept: ".tpi",
                onchange: move |evt| {
                    *small_state.write() = evt.value();
                }
            } 
        }
    }
}

#[component]
fn LoadBtn(state: Signal<Vec<Entry>>) -> Element {
    rsx! {
        p {
            "Load: "
            input {
                name: "load",
                r#type: "file",
                accept: ".tpi",
                onchange: move |evt| {
                    async move {
                        if let Some(file_engine) = &evt.files() {
                            if file_engine.files().len() > 0 {
                                let temp = serde_json::from_str(
                                    file_engine.read_file_to_string(
                                        file_engine.files()[0].as_str())
                                        .await.unwrap().as_str()).unwrap();
                                *state.write() = temp;
                            }
                        }
                    }
                },
            }
        }
    }
}

#[component]
fn GameControls(state: Signal<Vec<Entry>>) -> Element {
    let entries = state();
    rsx! {
        if entries.len() > 0 {
            NewBtn { state }
            SaveBtn { state }
        }
        LoadBtn { state }
    }
}
