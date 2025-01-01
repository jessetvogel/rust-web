
use crate::runtime::RuntimeFuture;
use crate::invoke::{Js, JsValue, ObjectRef};

use std::collections::HashMap;
use std::cell::RefCell;

thread_local! {
    pub static CALLBACK_HANDLERS: RefCell<HashMap<ObjectRef, Box<dyn FnMut(ObjectRef) + 'static>>> = Default::default();
}

pub fn create_callback(mut handler: impl FnMut(ObjectRef) + 'static) -> ObjectRef {
    let code = r#"
        const handler = (e) => {
            objects.push(e);
            const callbackObjectId = objects.length - 1;
            wasmModule.instance.exports.handle_callback(objectId,callbackObjectId);
        };
        objects.push(handler);
        const objectId = objects.length - 1;
        return objectId;
    "#;
    let object_id = Js::invoke(code, &[]).to_num().unwrap();
    let function_ref = ObjectRef::new(object_id as u32);
    let cb = move |value| { handler(value); };
    CALLBACK_HANDLERS.with(|s| { s.borrow_mut().insert(function_ref.clone(), Box::new(cb)); });
    function_ref
}

#[no_mangle]
pub fn handle_callback(callback_id: u32, param: i32) {

    let object_ref = ObjectRef::new(param as u32);

    CALLBACK_HANDLERS.with(|s| {
        let handler = s.borrow_mut().get_mut(&ObjectRef::new(callback_id)).unwrap() as *mut Box<dyn FnMut(_) + 'static>;
        unsafe { (*handler)(object_ref) }
    });

    Js::deallocate(object_ref);
}

pub fn create_async_callback() -> (ObjectRef, RuntimeFuture<ObjectRef>) {
    let future = RuntimeFuture::new();
    let future_id = future.id;
    let callback_ref = create_callback(move |e| { RuntimeFuture::wake(future_id, e); });
    return (callback_ref, future);
}

pub fn promise<F: FnOnce(ObjectRef) -> Vec<JsValue> + 'static>(code: &str, params_fn: F) -> RuntimeFuture<ObjectRef> {
    let (callback_ref, future) = create_async_callback();
    Js::invoke(code, &params_fn(callback_ref));
    future
}

#[cfg(test)]
mod tests {

    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_callback() {

        // add listener
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        create_callback(move |_| { *has_run_clone.borrow_mut() = true; });

        // simulate callback
        let function_ref = ObjectRef::new(0);
        handle_callback(*function_ref, 0);
        assert_eq!(*has_run.borrow(), true);

        // remove listener
        CALLBACK_HANDLERS.with(|s| { s.borrow_mut().remove(&function_ref); });
        let count = CALLBACK_HANDLERS.with(|s| s.borrow().len());
        assert_eq!(count, 0);
    }

    #[test]
    fn test_future_callback() {

        // add listener
        let (function_ref, future) = create_async_callback();

        // simulate callback
        handle_callback(*function_ref, 0);
        crate::runtime::Runtime::block_on(async move { future.await; });

        // remove listener
        CALLBACK_HANDLERS.with(|s| { s.borrow_mut().remove(&function_ref); });
        let count = CALLBACK_HANDLERS.with(|s| s.borrow().len());
        assert_eq!(count, 0);
    }

}
