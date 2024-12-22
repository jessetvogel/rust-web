
use tinyweb::callbacks::{create_callback, create_future_callback};
use tinyweb::runtime::{Runtime, RuntimeFuture};
use tinyweb::invoke::*;

#[no_mangle]
pub fn main() {

    std::panic::set_hook(Box::new(|e| { Js::invoke("console.log({})", &[Str(e.to_string())]); }));

    // invoke
    Js::invoke("console.log('invoke')", &[]);

    // callback
    let function_ref = create_callback(move |_s| { Js::invoke("console.log('invoke timer')", &[]); });
    Js::invoke("setTimeout({}, 1000)", &[ Ref(function_ref)]);

    // future callback
    let future = RuntimeFuture::<()>::new();
    let callback_ref = create_future_callback(future.id());
    Js::invoke("window.setTimeout({},2000)", &[Ref(callback_ref)]);
    Runtime::block_on(async move {
        future.await;
        Js::invoke("console.log('invoke timer future')", &[]);
    });
}
