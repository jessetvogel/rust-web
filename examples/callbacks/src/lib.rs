
use tinyweb::callbacks::{create_callback, create_future_callback};
use tinyweb::runtime::{Runtime, RuntimeFuture};
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[Str(e.to_string())]); }));

    // invoke
    Js::invoke("console.log('invoke')", &[]);

    // invoke callback
    let function_ref = create_callback(move |_s| { Js::invoke("console.log('invoke timer')", &[]); });
    Js::invoke("setTimeout({}, 1000)", &[ Ref(function_ref)]);

    // invoke future callback
    let future = RuntimeFuture::<()>::new();
    let callback_ref = create_future_callback(future.id());
    Js::invoke("window.setTimeout({},2000)", &[Ref(callback_ref)]);
    Runtime::block_on(async move {
        future.await;
        Js::invoke("console.log('invoke timer future')", &[]);
    });

    // invoke async callback
    let url = "https://pokeapi.co/api/v2/pokemon/1";
    let future = RuntimeFuture::<String>::new();
    let future_id = future.id();
    let callback_ref = create_callback(move |e| {
        let result = Js::invoke("return objects[{}].name", &[Str(e.to_string())]).to_str().unwrap();
        RuntimeFuture::wake(future_id, result);
    });
    Js::invoke("fetch({}).then(r => r.json()).then(r => { {}(r) })", &[Str(url.into()), Ref(callback_ref)]);
    Runtime::block_on(async move {
        Js::invoke("console.log('invoke fetch', {})", &[Str(future.await)]);
    });
}
