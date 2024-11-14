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

# Getting Started

### Create a new project

```rs
fn page1() -> El {
    El::new("div")
        .classes(&["m-2"])
        .child(El::new("button").text("page 1").classes(&BUTTON_CLASSES).on_event("click", move |_| {
            Js::invoke("console.log('hello browser')", &[]);
        }))
}

#[no_mangle]
pub fn main() {
    page1().mount(&body);
}
```

### Use the starter project

- Fork the [tinyweb-starter](https://github.com/LiveDuo/tinyweb-starter) project

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

TODO

# Credits

Credits to [Richard Anaya](https://github.com/richardanaya) for his work on [web.rs](https://github.com/richardanaya/web.rs) that provided solutions to some practical challanges in this library especially his work on [async support](https://github.com/richardanaya/web.rs/blob/master/crates/web/src/executor.rs). Also, to [Greg Johnston](https://github.com/gbj) for [his videos](https://www.youtube.com/@gbjxc/videos) that made working with signals in Rust easy.
