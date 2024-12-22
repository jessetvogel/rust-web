
mod keycodes;

use std::collections::HashMap;
use std::cell::RefCell;

use json::JsonValue;

use tinyweb::callbacks::{create_async_callback, promise};
use tinyweb::router::{Page, Router};
use tinyweb::runtime::Runtime;
use tinyweb::signals::Signal;
use tinyweb::element::El;

use tinyweb::invoke::*;

const BUTTON_CLASSES: &[&str] = &["bg-blue-500", "hover:bg-blue-700", "text-white", "p-2", "rounded", "m-2"];

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

async fn fetch_json(method: &str, url: &str, body: Option<JsonValue>) -> Result<JsonValue, String> {
    let body = body.map(|s| s.dump()).unwrap_or_default();
    let (callback_ref, future) = create_async_callback();
    let request = r#"
        const options = { method: {}, headers: { 'Content-Type': 'application/json' }, body: p0 !== 'GET' ? {} : null };
        fetch({}, options).then(r => r.json()).then(r => { {}(r) })
    "#;
    Js::invoke(request, &[Str(method.into()), Str(body), Str(url.into()), Ref(callback_ref)]);
    let object_id = future.await;
    let result = Js::invoke("return JSON.stringify(objects[{}])", &[Number(*object_id as f64)]).to_str().unwrap();
    json::parse(&result).map_err(|_| "Parse error".to_owned())
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
            let body = Js::invoke("return document.querySelector({})", &[Str("body".into())]).to_ref().unwrap();
            let signal_key_clone = signal_key_clone.clone();

            El::from(&body).on_event("keydown", move |e| {
                let key_code = Js::invoke("return {}[{}]", &[Ref(e), Str("keyCode".into())]).to_num().unwrap();
                let key_name = keycodes::KEYBOARD_MAP[key_code as usize];
                let text = format!("Pressed: {}", key_name);
                signal_key_clone.set(text);
            });

            // start timer
            let signal_time_clone = signal_time_clone.clone();
            Runtime::block_on(async move {
                loop {
                    signal_time_clone.set("⏰ tik");
                    promise("window.setTimeout({},{})", move |c| vec![Ref(c), Number(1_000.into())]).await;
                    signal_time_clone.set("⏰ tok");
                    promise("window.setTimeout({},{})", move |c| vec![Ref(c), Number(1_000.into())]).await;
                }
            });

        })
        .classes(&["m-2"])
        .child(El::new("button").text("api").classes(&BUTTON_CLASSES).on_event("click", |_| {
            Runtime::block_on(async move {
                let url = format!("https://pokeapi.co/api/v2/pokemon/{}", 1);
                let result = fetch_json("GET", &url, None).await.unwrap();
                let name = result["name"].as_str().unwrap();
                Js::invoke("alert({})", &[Str(name.into())]);
            });
        }))
        .child(El::new("button").text("page 2").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            ROUTER.with(|s| { s.borrow().navigate("/page2"); });
        }))
        .child(El::new("br"))
        .child(El::new("button").text("add").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            let count = signal_count_clone.get() + 1;
            signal_count_clone.set(count);
        }))
        .child(El::new("div").text("0").on_mount(move |el| {
            let el_clone = el.clone();
            signal_count.on(move |v| { Js::invoke("{}.innerHTML = {}", &[Ref(el_clone.element), Str(v.to_string())]); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_time.on(move |v| { Js::invoke("{}.innerHTML = {}", &[Ref(el_clone.element), Str(v.into())]); });
        }))
        .child(El::new("div").text("-").on_mount(move |el| {
            let el_clone = el.clone();
            signal_key.on(move |v| { Js::invoke("{}.innerHTML = {}", &[Ref(el_clone.element), Str(v.into())]); });
        }))
}

fn page2() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("page 1").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            ROUTER.with(|s| { s.borrow().navigate("/page1"); });
        }))
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[Str(e.to_string())]); }));

    // get pages
    let pages = [
        ("/page1".to_owned(), Page { element: page1(), title: None }),
        ("/page2".to_owned(), Page { element: page2(), title: None }),
        ("/".to_owned(), Page { element: page1(), title: None })
    ];

    // load page
    let body = Js::invoke("return document.querySelector({})", &[Str("body".into())]).to_ref().unwrap();
    let pathname = Js::invoke("return window.location.pathname", &[]).to_str().unwrap();
    let (_, page) = pages.iter().find(|&(s, _)| *s == pathname).unwrap_or(&pages[0]);
    page.element.mount(&body);

    // init router
    ROUTER.with(|s| {
        *s.borrow_mut() = Router { pages: HashMap::from_iter(pages), root: Some(body) };
    });

}
