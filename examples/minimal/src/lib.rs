
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {
    let body = Js::invoke_new("return document.querySelector({})", &[Str("body".into())]).to_ref().unwrap();
    Js::invoke_new("{}.innerHTML = {}", &[Ref(body), Str("hello".into())]);
}
