
use tinyweb::callbacks::create_callback;
use tinyweb::invoke::*;

pub fn add_click_event_listener(element: &ObjectRef, handler: impl FnMut(ObjectRef) + 'static) -> ObjectRef {

    let function_ref = create_callback(handler);
    Js::invoke("{}.addEventListener('click',{})", &[Ref(*element), Ref(function_ref)]);

    function_ref
}

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[Str(e.to_string())]); }));

    let button = Js::invoke("return document.createElement({})", &[Str("button".into())]).to_ref().unwrap();
    Js::invoke("{}.textContent = 'Click'", &[Ref(button)]);
    add_click_event_listener(&button, move |_s| { Js::invoke("alert('hello')", &[]); });

    let body = Js::invoke("return document.querySelector({})", &[Str("body".into())]).to_ref().unwrap();
    Js::invoke("{}.appendChild({})", &[Ref(body), Ref(button)]);
}
