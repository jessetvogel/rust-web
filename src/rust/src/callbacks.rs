
use crate::runtime::RuntimeFuture;
use crate::invoke::{Js, ObjectRef};

use std::collections::HashMap;
use std::sync::Mutex;

thread_local! {
    pub static CALLBACK_HANDLERS: Mutex<HashMap<ObjectRef, Box<dyn FnMut(ObjectRef) + 'static>>> = Default::default();
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
    CALLBACK_HANDLERS.with(|s| { s.lock().map(|mut s| { s.insert(function_ref.clone(), Box::new(cb)); }).unwrap(); });
    function_ref
}

#[no_mangle]
pub fn handle_callback(callback_id: u32, param: i32) {

    let object_ref = ObjectRef::new(param as u32);

    CALLBACK_HANDLERS.with(|s| {
        let handler = s.lock().map(|mut s| {
            s.get_mut(&ObjectRef::new(callback_id)).unwrap() as *mut Box<dyn FnMut(_) + 'static>
        }).unwrap();
        unsafe { (*handler)(object_ref) }
    });

    Js::deallocate(object_ref);
}

pub fn create_async_callback() -> (ObjectRef, RuntimeFuture<ObjectRef>) {
    let future = RuntimeFuture::<ObjectRef>::new();
    let future_id = future.id();
    let callback_ref = create_callback(move |e| { RuntimeFuture::wake(future_id, e); });
    return (callback_ref, future);
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
        CALLBACK_HANDLERS.with(|s| { s.lock().map(|mut s| { s.remove(&function_ref); }).unwrap(); });
        let count = CALLBACK_HANDLERS.with(|s| s.lock().map(|s| s.len()).unwrap());
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
        CALLBACK_HANDLERS.with(|s| { s.lock().map(|mut s| { s.remove(&function_ref); }).unwrap(); });
        let count = CALLBACK_HANDLERS.with(|s| s.lock().map(|s| s.len()).unwrap());
        assert_eq!(count, 0);
    }

}
