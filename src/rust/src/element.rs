use crate::{
    callbacks::add_event_listener,
    console_error,
    js::{self, ObjectRef},
};
use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Elem {
    pub element: ObjectRef,
    pub callbacks: RefCell<Vec<ObjectRef>>,
}

impl Elem {
    pub fn new(tag: &str) -> Self {
        let element = match js::create_element(tag).to_ref() {
            Ok(r) => r,
            Err(_) => {
                console_error!("Failed to created element with tag '{}'", tag,);
                panic!();
            }
        };
        Self {
            element,
            callbacks: RefCell::new(vec![]),
        }
    }

    pub fn select(query: &str) -> Result<Self, &'static str> {
        match js::query_selector(query).to_ref() {
            Ok(r) => Ok(Self::from(r)),
            Err(_) => Err("query did not match any element"),
        }
    }

    pub fn append(self, child: &Elem) -> Self {
        js::invoke(
            "{}.append({})",
            &[self.element.clone().into(), child.element.clone().into()],
        );
        self
    }

    pub fn remove(self) {
        js::invoke("{}.remove()", &[self.element.into()]);
    }

    pub fn attr(self, name: &str, value: &str) -> Self {
        js::invoke(
            "{}.setAttribute({},{})",
            &[self.element.clone().into(), name.into(), value.into()],
        );
        self
    }

    pub fn class(self, class: &str) -> Self {
        js::invoke(
            "{}.classList.add(...{}.split(' '))",
            &[self.element.clone().into(), class.into()],
        );
        self
    }

    pub fn children(self, children: &[&Self]) -> Self {
        js::invoke(
            "{}.innerHTML = {}",
            &[self.element.clone().into(), "".into()],
        );
        for child in children {
            js::invoke(
                "{}.appendChild({})",
                &[self.element.clone().into(), child.element.clone().into()],
            );
        }
        self
    }

    pub fn on(self, event: &str, callback: impl FnMut(ObjectRef) + 'static) -> Self {
        add_event_listener(&self.element, event, callback);
        self
    }

    pub fn text(self, text: &str) -> Self {
        let text = js::invoke("return document.createTextNode({})", &[text.into()])
            .to_ref()
            .unwrap();
        js::invoke(
            "{}.appendChild({})",
            &[self.element.clone().into(), text.into()],
        );
        self
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

impl From<ObjectRef> for Elem {
    fn from(value: ObjectRef) -> Self {
        Self {
            element: value,
            callbacks: RefCell::new(vec![]),
        }
    }
}
