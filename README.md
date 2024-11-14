# TinyWeb 🔌 Rust on the client. No dependencies.

Build the client-side with Rust. Compbine it with any http framework to build fullstack applications!

# What's TinyWeb?

TinyWeb is a toolkit to build web applications that care about simplicity and correctness.

Aims to solve robustness with using Rust's strict type system, zero-cost abstractions and great built-in tooling.

Aims to sove simplicity with its tiny footprint (< 800 lines of Rust), it's design with no build step and by having no external dependencies.

# Features

- No Javascript
- No macros
- No dependencies
- No build step
- Just HTML & Rust (Wasm)

**Note:** No build step besides `cargo build` and `cp`

# Getting Started

### Use the starter project

- Fork the [tinyweb-starter](https://github.com/LiveDuo/tinyweb-starter) project

### Create a new project

```rs
fn component() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("page 1").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            Js::invoke("alert('hello browser')", &[]);
        }))
}

#[no_mangle]
pub fn main() {
    component().mount(&body);
}
```

# How it works

TODO

# How to's & guides

### Router support

TODO

### Reactivity and Signals

TODO

### Browser APIs

TODO

### Async Support

TODO

# Backstory

For quite some time, I've been torn about typescript.

One on hand, it brings stronger typing to javascript improving correctness.

One the other hand, it comes with a heavy build system with heavy cost on simplicity.

While undecided, I had to build something that really relied on correctness, a financial application, and realized that how much I don't trust typescript even what's design to do.

I then tried different wasm based frameworks like Leptos and Yew. While great at correctness, they require hundereds of dependencies just to get started. After digging more into it, I realised that all these dependencies come from `wasm-bindgen` that's maintained by [The Rust and WebAssembly Working Group](https://rustwasm.github.io).

The `wasm-bindgen` crate is great, it focuses on performance and has bindings for most browser APIs but that came at a cost through the number of dependencies it requires. For reference, leptos development tool `cargo-leptos` depends on other 485 crates and `leptos` itself on 231 more.

So, I setup out to build a web framework that aims for both simplicity and correctness, one that's based on Rust but has no dependencies.


# Credits

Credits to [Richard Anaya](https://github.com/richardanaya) for his work on [web.rs](https://github.com/richardanaya/web.rs) that provided solutions to some practical challanges in this library especially his work on [async support](https://github.com/richardanaya/web.rs/blob/master/crates/web/src/executor.rs). Also, to [Greg Johnston](https://github.com/gbj) for [his videos](https://www.youtube.com/@gbjxc/videos) that made working with signals in Rust easy.
