
use tinyweb::handlers::create_callback;
use tinyweb::invoke::*;

pub fn add_click_event_listener(element: &ObjectRef, handler: impl FnMut(ObjectRef) + 'static) -> ObjectRef {

    let function_ref = create_callback(handler);
    Js::invoke_new("{}.addEventListener('click',{})", &[Ref(*element), Ref(function_ref)]);

    function_ref
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke_new("console.log({})", &[Str(e.to_string())]); }));

    let button = Js::invoke_ref("return document.createElement({})", &[Str("button".into())]);
    let button_text = Js::invoke_ref("return document.createTextNode({})", &[Str("Click".into())]);
    Js::invoke_new("{}.appendChild({})", &[Ref(button), Ref(button_text)]);
    add_click_event_listener(&button, move |_s| {
        let button = Js::invoke_ref("return document.createElement({})", &[Str("button".into())]);
        add_click_event_listener(&button, |_s| {});
    });

    let body = Js::invoke_ref("return document.querySelector({})", &[Str("body".into())]);
    Js::invoke_new("{}.appendChild({})", &[Ref(body), Ref(button)]);
}
