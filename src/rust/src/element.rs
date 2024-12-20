
use std::cell::RefCell;
use std::ops::Deref;

use crate::invoke::{Js, ObjectRef};

use crate::invoke::JsValue::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct El { element: ObjectRef, callbacks: RefCell<Vec<ObjectRef>> }

impl Deref for El {
    type Target = ObjectRef;

    fn deref(&self) -> &Self::Target {
        &self.element
    }
}

impl El {
    pub fn new(tag: &str) -> Self {
        let el = Js::invoke("return document.createElement({})", &[Str(tag.into())]).to_ref().unwrap();
        Self { element: el, callbacks: RefCell::new(vec![]) }
    }
    pub fn from(el: &ObjectRef) -> Self {
        Self { element: el.to_owned(), callbacks: RefCell::new(vec![]) }
    }
    pub fn mount(&self, parent: &ObjectRef) {
        Js::invoke("{}.appendChild({})", &[Ref(*parent), Ref(self.element)]);
    }
    pub fn unmount(&self) {
        let mut c = self.callbacks.borrow_mut();
        c.iter().for_each(|p| {
            crate::handlers::CALLBACK_HANDLERS.with(|s| {
                s.lock().map(|mut h| { let _ = h.remove(p).unwrap(); }).unwrap();
            });
        });
        c.clear();
    }
    pub fn attr(self, name: &str, value: &str) -> Self {
        Js::invoke("{}.setAttribute({},{})", &[Ref(*self), Str(name.into()), Str(value.into())]);
        self
    }
    pub fn attr_fn(self, name: &str, value: &str, cb: impl Fn() -> bool + 'static) -> Self {
        if cb() {
            Js::invoke("{}.setAttribute({},{})", &[Ref(*self), Str(name.into()), Str(value.into())]);
        }
        self
    }
    pub fn classes(self, classes: &[&str]) -> Self {
        classes.iter().for_each(|&c| { Js::invoke("{}.classList.add({})", &[Ref(*self), Str(c.into())]); });
        self
    }
    pub fn child(self, child: Self) -> Self {
        Js::invoke("{}.appendChild({})", &[Ref(*self), Ref(*child)]);
        self
    }
    pub fn children(self, children: &[Self]) -> Self {
        Js::invoke("{}.innerHTML = {}", &[Ref(*self), Str("".into())]);
        for child in children {
            Js::invoke("{}.appendChild({})", &[Ref(*self), Ref(child.element)]);
        }
        self
    }
    pub fn on_mount(self, mut cb: impl FnMut(&Self) + 'static) -> Self {
        cb(&self);
        self
    }
    pub fn on_event(self, event: &str, cb: impl FnMut(ObjectRef) + 'static) -> Self {

        let function_ref = crate::handlers::create_callback(cb);
        let code = &format!("{{}}.addEventListener('{}',{{}})", event);
        Js::invoke(code, &[Ref(*self), Ref(function_ref)]);

        self.callbacks.borrow_mut().push(function_ref);

        self
    }
    pub fn text(self, text: &str) -> Self {

        let el = Js::invoke("return document.createTextNode({})", &[Str(text.into())]).to_ref().unwrap();
        Js::invoke("{}.appendChild({})", &[Ref(*self), Ref(el)]);

        self
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_element() {

        let el = El::new("div").classes(&[])
            .child(El::new("button").text("button 1"))
            .child(El::new("button").text("button 2"));
        assert_eq!(el, El::from(&ObjectRef::new(0)));

    }

}
