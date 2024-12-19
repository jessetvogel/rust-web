

use std::collections::HashMap;

use crate::invoke::{Js, ObjectRef};
use crate::element::El;

use crate::invoke::InvokeParam::*;

#[derive(Debug)]
pub struct Page { pub element: El, pub title: Option<String> }

#[derive(Debug, Default)]
pub struct Router { pub root: Option<ObjectRef>, pub pages: HashMap::<String, Page> }

impl Router {
    pub fn navigate(&self, route: &str) {

        // unmount page
        let pathname = Js::invoke("return window.location.pathname", &[]).to_str().unwrap();
        let (_, current_page) = self.pages.iter().find(|&(s, _)| *s == pathname).unwrap();
        current_page.element.unmount();

        // push state
        let page = self.pages.get(route).unwrap();
        let page_str = page.title.to_owned().unwrap_or_default();
        Js::invoke("window.history.pushState({ },{},{})", &[Str(page_str.into()), Str(route.into())]);

        // mount new page
        let body = self.root.as_ref().unwrap();
        Js::invoke("{}.innerHTML = {}", &[Ref(*body), Str("".into())]);
        page.element.mount(&body);
    }
}
