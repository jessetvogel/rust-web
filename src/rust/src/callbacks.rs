use std::{cell::RefCell, collections::HashMap};

use crate::{console_error, js, js::ObjectRef};

thread_local! {
    // Hashmap used to store all callbacks.
    static CALLBACKS: RefCell<HashMap<u32, Box<dyn FnMut(ObjectRef) + 'static>>> = RefCell::new(HashMap::new());
}

extern "C" {
    fn __add_event_listener(object_id: u32, e_ptr: *const u8, e_len: u32, callback_id: u32);
}

#[no_mangle]
pub fn call_callback(id: u32, event_id: u32) {
    CALLBACKS.with_borrow_mut(|map| match map.get_mut(&id) {
        None => {
            console_error!("could not find callback with id {}", id);
        }
        Some(f) => (*f)(ObjectRef::new(event_id)),
    })
}

pub fn add_event_listener(
    object: &ObjectRef,
    event: &str,
    callback: impl FnMut(ObjectRef) + 'static,
) {
    // Store callback with a new id
    let callback_id = CALLBACKS.with_borrow_mut(|map| {
        let id = map.len() as u32;
        map.insert(id, Box::new(callback));
        id
    });

    // Call JS
    unsafe {
        __add_event_listener(object.id(), event.as_ptr(), event.len() as u32, callback_id);
    }
}
