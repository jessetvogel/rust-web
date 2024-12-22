
use tinyweb::callbacks::{create_async_callback, create_callback, create_future_callback};
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
    let (callback_ref, future) = create_async_callback();
    Js::invoke("fetch({}).then(r => r.json()).then(r => { {}(r) })", &[Str(url.into()), Ref(callback_ref)]);
    Runtime::block_on(async move {
        let object_id = future.await;
        let result = Js::invoke("return objects[{}].name", &[Number(*object_id as f64)]).to_str().unwrap();
        Js::invoke("console.log('invoke fetch', {})", &[Str(result)]);
    });
}
