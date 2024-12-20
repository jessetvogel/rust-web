
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {
    let body = Js::invoke("return document.querySelector({})", &[Str("body".into())]).to_ref().unwrap();
    Js::invoke("{}.innerHTML = {}", &[Ref(body), Str("hello".into())]);
}
