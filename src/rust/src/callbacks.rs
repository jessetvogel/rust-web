
use crate::runtime::RuntimeFuture;
use crate::invoke::{Js, ObjectRef};

use crate::invoke::JsValue::*;

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
    insert_callback(function_ref, move |value| { handler(value); });
    function_ref
}

pub fn create_future_callback(future_id: u32) -> ObjectRef {
    Js::invoke("return () => { wasmModule.instance.exports.handle_future_callback({}); }", &[Number(future_id as f64)]).to_ref().unwrap()
}

pub fn insert_callback(function_ref: ObjectRef, cb: impl FnMut(ObjectRef) + 'static) {
    CALLBACK_HANDLERS.with(|s| { s.lock().map(|mut s| { s.insert(function_ref.clone(), Box::new(cb)); }).unwrap(); });
}

pub fn remove_callback(function_ref: ObjectRef) {
    CALLBACK_HANDLERS.with(|s| { s.lock().map(|mut s| { s.remove(&function_ref); }).unwrap(); });
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

#[no_mangle]
pub fn handle_future_callback(callback_id: u32) {
    RuntimeFuture::wake(callback_id, ());
}

#[cfg(test)]
mod tests {

    use std::cell::RefCell;
    use std::rc::Rc;

    use super::*;

    #[test]
    fn test_handler() {

        let function_ref = ObjectRef::new(0);

        // add listener
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        insert_callback(function_ref, move |_| { *has_run_clone.borrow_mut() = true; });

        // call listener
        handle_callback(*function_ref, 0);
        assert_eq!(*has_run.borrow(), true);

        // remove listener
        remove_callback(function_ref);
        let count = CALLBACK_HANDLERS.with(|s| s.lock().map(|s| s.len()).unwrap());
        assert_eq!(count, 0);
    }

}
