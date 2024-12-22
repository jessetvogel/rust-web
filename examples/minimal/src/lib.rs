
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {
    let body = Js::invoke("return document.querySelector({})", &["body".into()]).to_ref().unwrap();
    Js::invoke("{}.innerHTML = {}", &[body.into(), "hello".into()]);
}
