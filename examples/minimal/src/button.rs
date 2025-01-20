use web::{components::Component, element::Elem};

pub struct Button {
    elem: Elem,
}

impl Button {
    pub fn new(text: &str) -> Self {
        Self {
            elem: Elem::new("button")
                .classes(&[
                    "text-blue-700",
                    "hover:text-white",
                    "border",
                    "border-blue-700",
                    "hover:bg-blue-800",
                    "focus:ring-4",
                    "focus:outline-none",
                    "focus:ring-blue-300",
                    "font-medium",
                    "rounded-lg",
                    "text-sm",
                    "px-5",
                    "py-2.5",
                    "text-center",
                    "me-2",
                    "mb-2",
                    "dark:border-blue-500",
                    "dark:text-blue-500",
                    "dark:hover:text-white",
                    "dark:hover:bg-blue-500",
                    "dark:focus:ring-blue-800",
                ])
                .text(text),
        }
    }
}

impl Component for Button {
    fn to_elem(&self) -> &Elem {
        &self.elem
    }
}
