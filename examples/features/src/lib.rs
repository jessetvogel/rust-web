
mod keycodes;

use std::collections::HashMap;
use std::future::Future;
use std::cell::RefCell;

use json::JsonValue;

use tinyweb::handlers::create_future_callback;
use tinyweb::runtime::{Runtime, RuntimeFuture};
use tinyweb::router::{Page, Router};
use tinyweb::signals::Signal;
use tinyweb::element::El;

use tinyweb::http::*;
use tinyweb::invoke::*;

const BUTTON_CLASSES: &[&str] = &["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"];

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

async fn fetch_json(method: HttpMethod, url: String, body: Option<JsonValue>) -> JsonValue {
    let body_temp = body.map(|s| s.dump());
    let body = body_temp.as_ref().map(|s| s.as_str());
    let fetch_options = FetchOptions { method, url: &url, body, ..Default::default()};
    let fetch_res = fetch(fetch_options).await;
    let result = match fetch_res { FetchResponse::Text(_, d) => Ok(d), _ => Err(()), };
    json::parse(&result.unwrap()).unwrap()
}

pub fn sleep(ms: impl Into<f64>) -> impl Future<Output = ()> {
    let future = RuntimeFuture::new();
    let callback_ref = create_future_callback(future.id());
    Js::invoke("window.setTimeout({},{})", &[Ref(&callback_ref), Number(ms.into())]);
    future
}

fn page1() -> El {

    // key signal
    let signal_key = Signal::new("-".to_owned());
    let signal_key_clone = signal_key.clone();

    // count signal
    let signal_count = Signal::new(0);
    let signal_count_clone = signal_count.clone();

    // time signal
    let signal_time = Signal::new("-");
    let signal_time_clone = signal_time.clone();

    El::new("div")
        .on_mount(move |_| {

            // add listener
            let body = Js::invoke_ref("return document.querySelector({})", &[Str("body")]);
            let signal_key_clone = signal_key_clone.clone();

            El::from(&body).on_event("keydown", move |e| {
                let key_code = Js::invoke_number("return {}[{}]", &[Ref(&e), Str("key_code")]);
                let key_name = keycodes::KEYBOARD_MAP[key_code as usize];
                let text = format!("Pressed: {}", key_name);
                signal_key_clone.set(text);
            });

            // start timer
            let signal_time_clone = signal_time_clone.clone();
            Runtime::block_on(async move {
                loop {
                    signal_time_clone.set("⏰ tik");
                    sleep(1_000).await;
                    signal_time_clone.set("⏰ tok");
                    sleep(1_000).await;
                }
            });

        })
        .classes(&["m-2"])
        .child(El::new("button").text("api").classes(&BUTTON_CLASSES).on_event("click", |_| {
            Runtime::block_on(async move {
                let url = format!("https://pokeapi.co/api/v2/pokemon/{}", 1);
                let result = fetch_json(HttpMethod::GET, url, None).await;
                let name = result["name"].as_str().unwrap();
                Js::invoke("alert({})", &[Str(&name.to_owned())]);
            });
        }))
        .child(El::new("button").text("page 2").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            ROUTER.with(|s| { s.borrow().navigate("page2"); });
        }))
        .child(El::new("br"))
        .child(El::new("button").text("add").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            let count = signal_count_clone.get() + 1;
            signal_count_clone.set(count);
        }))
        .child(El::new("div").text("0").on_mount(move |el| {
            let el_clone = el.clone();
            signal_count.on(move |v| { Js::invoke("{}.innerHTML = {}", &[Ref(&el_clone), Str(&v.to_string())]); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_time.on(move |v| { Js::invoke("{}.innerHTML = {}", &[Ref(&el_clone), Str(&v)]); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_key.on(move |v| { Js::invoke("{}.innerHTML = {}", &[Ref(&el_clone), Str(&v)]); });
        }))
}

fn page2() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("page 1").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            ROUTER.with(|s| { s.borrow().navigate("page1"); });
        }))
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[Str(&e.to_string())]); }));

    // get pages
    let pages = [
        ("page1".to_owned(), Page { element: page1(), title: None }),
        ("page2".to_owned(), Page { element: page2(), title: None })
    ];

    // load page
    let body = Js::invoke_ref("return document.querySelector({})", &[Str("body")]);
    let pathname = Js::invoke_str("return window.location.pathname", &[]);
    let (_, page) = pages.iter().find(|&(s, _)| *s == pathname).unwrap_or(&pages[0]);
    page.element.mount(&body);

    // init router
    ROUTER.with(|s| {
        *s.borrow_mut() = Router { pages: HashMap::from_iter(pages), root: Some(body) };
    });

}
