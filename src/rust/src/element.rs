use crate::js::{Js, ObjectRef};
use std::cell::RefCell;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Elem {
    pub element: ObjectRef,
    pub callbacks: RefCell<Vec<ObjectRef>>,
}

impl Elem {
    pub fn new(tag: &str) -> Self {
        let elem = Js::invoke("return document.createElement({})", &[tag.into()])
            .to_ref()
            .unwrap();
        Self {
            element: elem,
            callbacks: RefCell::new(vec![]),
        }
    }

    pub fn append(&self, child: &Elem) -> &Self {
        Js::invoke(
            "{}.append({})",
            &[self.element.clone().into(), child.element.clone().into()],
        );
        &self
    }

    pub fn remove(self) {
        Js::invoke("{}.remove()", &[self.element.into()]);
    }

    pub fn attr(&self, name: &str, value: &str) -> &Self {
        Js::invoke(
            "{}.setAttribute({},{})",
            &[self.element.clone().into(), name.into(), value.into()],
        );
        &self
    }

    pub fn classes(&self, classes: &[&str]) -> &Self {
        classes.iter().for_each(|&c| {
            Js::invoke(
                "{}.classList.add({})",
                &[self.element.clone().into(), c.into()],
            );
        });
        &self
    }

    pub fn children(&self, children: &[Self]) -> &Self {
        Js::invoke(
            "{}.innerHTML = {}",
            &[self.element.clone().into(), "".into()],
        );
        for child in children {
            Js::invoke(
                "{}.appendChild({})",
                &[self.element.clone().into(), child.element.clone().into()],
            );
        }
        &self
    }

    // pub fn on(&self, event: &str, cb: impl FnMut(ObjectRef) + 'static) -> &Self {
    //     let function_ref = crate::callbacks::create_callback(cb);
    //     let code = &format!("{{}}.addEventListener('{}',{{}})", event);
    //     Js::invoke(
    //         code,
    //         &[self.element.clone().into(), function_ref.clone().into()],
    //     );
    //     self.callbacks.borrow_mut().push(function_ref);
    //     &self
    // }

    pub fn text(&self, text: &str) -> &Self {
        let el = Js::invoke("return document.createTextNode({})", &[text.into()])
            .to_ref()
            .unwrap();
        Js::invoke(
            "{}.appendChild({})",
            &[self.element.clone().into(), el.into()],
        );

        &self
    }
}

impl From<&ObjectRef> for Elem {
    fn from(value: &ObjectRef) -> Self {
        Self {
            element: value.to_owned(),
            callbacks: RefCell::new(vec![]),
        }
    }
}
