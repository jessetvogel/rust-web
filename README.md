# TinyWeb 🌱 Rust on the client. No dependencies.

Build the client-side with Rust. Works with any http framework to build fullstack applications in pure Rust!

# What's TinyWeb?

TinyWeb is a toolkit to build web applications that care about both correctness and simplicity.

Allows client side applications to be build in pure Rust in a similar fashion to backend applications, utilizing the language strict type system and great built-in tooling. Has a tiny footprint with less than 800 lines of code, has no build step and no external dependencies.

# Features

- No Javascript
- No macros
- No dependencies
- No build step
- Just HTML & Rust (Wasm)

**Note:** No build step besides `cargo build`

# Getting Started

### Use the starter project

- Fork the [tinyweb-starter](https://github.com/LiveDuo/tinyweb-starter) project

### Create a new project

Create a new Rust project with `cargo new tinyweb-example --lib`:

Update the `src/lib.rs`:
```rs
use tinyweb::element::El;
use tinyweb::invoke::Js;

fn component() -> El {
    El::new("div")
        .child(El::new("button").text("print").on_event("click", move |_| {
            Js::invoke("alert('hello browser')", &[]);
        }))
}

#[no_mangle]
pub fn main() {
    let body = Js::invoke("return document.querySelector('body')"]).to_ref().unwrap();
    component().mount(&body);
}
```

Then, create an `index.html` in a new `public` folder:
```html
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <script src="https://cdn.jsdelivr.net/gh/LiveDuo/tinyweb/src/js/main.js"></script>
        <script type="application/wasm" src="client.wasm"></script>
    </head>
    <body></body>
</html>
```

Lastly, add `crate-type =["cdylib"]` right below `[lib]` section in `Cargo.toml`.

Then build the project with `cargo build --target wasm32-unknown-unknown -r` and `cp target/wasm32-unknown-unknown/release/*.wasm public/client.wasm` to get the `.wasm` in the right place. You can now serve the contents of the `public` folder with your favourite static http server.



# How it works

Each project built with TinyWeb has 3 components, an `index.html`, a static `main.js` and a `client.wasm` file compile from Rust with `cargo build --target wasm32-unknown-unknown -r`. These files can be served using an static HTTP server.

**Init:**
So when a user opens the website `index.html` is loaded in the browser which loads [main.js](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js). This file registers a [DOMContentLoaded](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js) event which when triggered [calls](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js) the `main` function in the wasm file. The `main` function usually makes the initial DOM rendering and registers listeners for different DOM events.

**Browser APIs and callbacks:**
Every time a rust function wants to invoke a browser API it uses the [__invoke](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/rust/src/invoke.rs) function under the hood which calls the [homonymous function](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js) in Javascript. When a previously registered callback is triggered a function named [handle_callback](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/rust/src/handlers.rs) in called that handles the callback logic.

*Note:* One key difference between TinyWeb and most other Rust based web frameworks is that there isn't a build step to compile browser bindings for Rust / Javascript using `wasm-bindgen`. Instead this library only supports primitive Javascript types such as numbers, booleans, bigints, strings, buffers and objects references to simplify the building process and have a static javascript file.

# How to's & guides

### Browser APIs

```rs
use tinyweb::invoke::Js;

Js::invoke("alert('hello browser')", &[]);
```

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs)

### Reactivity and Signals

```rs

use tinyweb::signals::Signal;
use tinyweb::element::El;

let signal_count = Signal::new(0);

El::new("button").text("add").on_event("click", move |_| {
    let count = signal_count.get() + 1;
    signal_count.set(count);
});
```

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs)

### Router support

```rs
use tinyweb::router::{Page, Router};

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

// initialize router
let pages = &[Page::new("/page1", page_component(), None)];
ROUTER.with(|s| { *s.borrow_mut() = Router::new("body", pages); });

// navigate to route
ROUTER.with(|s| { s.borrow().navigate("/page1"); });
```

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs)

### Async Support

```rs
use tinyweb::runtime::Runtime;
use tinyweb::invoke::Js;

Runtime::block_on(async move {

    let (cb, future) = create_async_callback();
    Js::invoke("setTimeout({}, 1000)", &[cb.into()]);
    future.await;

    Js::invoke("alert('timer')");
});
```

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs)

# Backstory

For quite some time, I couldn't decide if I like Typescript or not. On one hand, it has stronger typing than pure Javascript bringing more confidence about the code but on the other hand it comes with a heavy build system that makes complicates the project a lot and makes debugging a significantly harder.

When I had to build a application where I really cared that correctness, I realized how much I don't trust Typescript even for what's designed to do and I tried different wasm based web frameworks that allow building web applications using Rust. These frameworks alleviated the correctness concern but they complicate things a lot requiring hundereds of dependencies just to get started. For reference, `leptos` depends on 231 crates and its development tool `cargo-leptos` depends on another 485 crates.

A major reason for this complexity is that these frameworks depend on the `wasm-bindgen` crate that build the Rust bindings for browser APIs and the Javascript glue code that allows making these calls. But the `wasm-bindgen` is just one way to interact with browser APIs that trades off simplicity for performance and not all applications benefit from this trade off. The application I building most probably wouldn't.

So, I set out to build a web framework that allows to build client side applications with Rust that has a minimal footprint. The result is `TinyWeb`, a client side Rust framework built in <800 lines of code.

# Credits

Credits to [Richard Anaya](https://github.com/richardanaya) for his work on [web.rs](https://github.com/richardanaya/web.rs) that provided ideas to practical challenges on [async support](https://github.com/richardanaya/web.rs/blob/master/crates/web/src/executor.rs). Also, to [Greg Johnston](https://github.com/gbj) for [his videos](https://www.youtube.com/@gbjxc/videos) that show how to use Solid.js-like signals in Rust.
