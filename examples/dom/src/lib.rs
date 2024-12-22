
use tinyweb::callbacks::create_callback;
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[Str(e.to_string())]); }));

    let button = Js::invoke("return document.createElement({})", &[Str("button".into())]).to_ref().unwrap();
    Js::invoke("{}.textContent = 'Click'", &[Ref(button)]);

    let function_ref = create_callback(move |_s| { Js::invoke("alert('hello')", &[]); });
    Js::invoke("{}.addEventListener('click',{})", &[Ref(button), Ref(function_ref)]);

    let body = Js::invoke("return document.querySelector({})", &[Str("body".into())]).to_ref().unwrap();
    Js::invoke("{}.appendChild({})", &[Ref(body), Ref(button)]);
}
