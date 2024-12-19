
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {
    let body = Js::invoke_ref("return document.querySelector({})", &[Str("body".into())]);
    Js::invoke_new("{}.innerHTML = {}", &[Ref(body), Str("hello".into())]);
}
