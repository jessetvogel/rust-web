use button::Button;
use web::{
    components::Component,
    console_log,
    element::Elem,
    js::{self, JsValue},
};

mod button;

#[no_mangle]
pub fn main() {
    // Test all data types
    js::invoke(
        "console.log('[RUST]', {}, {}, {}, {}, {}, {}, {}, {})",
        &[
            JsValue::Undefined,
            JsValue::Null,
            true.into(),
            false.into(),
            1.23.into(),
            JsValue::BigInt(1234),
            "Hello!".into(),
            vec![0x12, 0x34, 0x56].into(),
        ],
    );

    // Create some layout
    let body = Elem::select("body").unwrap();

    console_log!("[RUST] body id = {}", body.element.id());

    let body = body.class("bg-gray-800");

    let body = body.append(
        &Elem::new("div")
            .class("flex flex-row gap-40 w-screen justify-around p-4")
            .children(&[
                &Button::new("Click me!").to_elem(),
                Button::new("No, click me!").to_elem(),
                Button::new("Or me!").to_elem(),
            ]),
    );

    let text = Elem::new("span").class("text-red-800");

    let body = body.append(&text);

    let input = Elem::new("input").class("m-8").on("input", move |event| {
        let value = js::invoke("return {}.target.value", &[event.into()])
            .to_string()
            .unwrap();
        text.clone().children(&[&Elem::new("span").text(&value)]);
    });

    body.append(&input);
}
