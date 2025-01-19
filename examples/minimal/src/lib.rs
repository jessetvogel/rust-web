use web::{
    console_log,
    js::{self, JsValue},
};

#[no_mangle]
pub fn main() {
    let body = js::query_selector("body").to_ref().unwrap();

    console_log!("[RUST] body id = {}", body.id());

    js::invoke(
        "{}.innerHTML = {}",
        &[body.into(), "Hello world from Rust!".into()],
    );

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
}
