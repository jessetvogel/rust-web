# TinyWeb 🌱 Rust on the client. No dependencies.

Build the client-side with Rust. Compbine it with any http framework to build fullstack applications!

# What's TinyWeb?

TinyWeb is a toolkit to build web applications that care about simplicity and correctness.

Aims to solve robustness by using Rust's strict type system, zero-cost abstractions and great built-in tooling.

Aims to sove simplicity with its tiny footprint (< 800 lines of Rust) and by having no build step and no external dependencies.

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
    let body = Js::invoke_ref("return document.querySelector('body')"]);
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

At first, the Rust code is compiled to wasm with `cargo build --target wasm32-unknown-unknown -r` and has to be serve alongside `index.html`. Once `index.html` is loaded in the browser, a [DOMContentLoaded](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js#L114) event
is triggered in [main.js](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js#L91) which loads the wasm file. Note that, in contrast to other Rust based frameworks, the javascript file here is static. That's because it does not use `wasm-bindgen` to build the browser bindings but instead the only types that are passed to and from javascript are primitive types such as numbers, strings, buffers and references to javascript objects.

Once the wasm file is loaded, the `main` function is [called](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js#L96) and this function acts as an initialization hook similar to `DOMContentLoaded` in vanilla javascript or `useEffect` in React. The `main` function usually makes the initial rendering and registers listeners for different DOM elements.

Every time a rust function wants to invoke a browser API it uses the [__invoke_and_return](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/rust/src/invoke.rs#L84) which calls the [corresponding function](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/js/main.js#L64) in javascript. Callbacks such as event listeners are register through the `__invoke_and_return` function and then call a dedicated function in wasm named [handle_callback](https://github.com/LiveDuo/tinyweb/blob/feature/readme/src/rust/src/handlers.rs#L14).

# How to's & guides

### Index html



Check it out [here](https://github.com/LiveDuo/tinyweb-starter/blob/master/public/index.html)

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

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs#L94)

### Browser APIs

```rs
use tinyweb::invoke::Js;

Js::invoke("alert('hello browser')", &[]);
```

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs#L87)

### Router support

```rs
use tinyweb::router::{Page, Router};

thread_local! {
    pub static ROUTER: RefCell<Router> = RefCell::new(Router::default());
}

ROUTER.with(|s| { s.borrow().navigate("page1"); });
```

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs#L21)

### Async Support

```rs
use tinyweb::http::{fetch, FetchResponse, FetchOptions};
use tinyweb::runtime::Runtime;
use tinyweb::invoke::Js;

Runtime::block_on(async move {
    let url = "https://pokeapi.co/api/v2/pokemon/1";
    let fetch_options = FetchOptions { method: HttpMethod::GET, url, ..Default::default()};
    let result = fetch(fetch_options).await;
    let result_text = match fetch_res { FetchResponse::Text(_, d) => Ok(d), _ => Err(()), };
    let result_json = json::parse(&result_text.unwrap()).unwrap();
    let name = result_json["name"].as_str().unwrap();
    Js::invoke("alert({})", &[Str(&name.to_owned())]);
});
```

Check it out [here](https://github.com/LiveDuo/tinyweb/blob/feature/readme/examples/features/src/lib.rs#L83)

# Backstory

For quite some time, I've been torn about typescript.

One on hand, it brings stronger typing to javascript improving correctness.

One the other hand, it comes with a heavy build system with heavy cost on simplicity.

While undecided, I had to build something that really relied on correctness, a financial application, and realized that how much I don't trust typescript even what's design to do.

I then tried different wasm based frameworks like Leptos and Yew. While great at correctness, they require hundereds of dependencies just to get started. After digging more into it, I realised that all these dependencies come from `wasm-bindgen` that's maintained by [The Rust and WebAssembly Working Group](https://rustwasm.github.io).

The `wasm-bindgen` crate is great, it focuses on performance and has bindings for most browser APIs but that came at a cost through the number of dependencies it requires. For reference, leptos development tool `cargo-leptos` depends on other 485 crates and `leptos` itself on 231 more.

So, I setup out to build a web framework that aims for both simplicity and correctness, one that's based on Rust but has no dependencies.


# Credits

Credits to [Richard Anaya](https://github.com/richardanaya) for his work on [web.rs](https://github.com/richardanaya/web.rs) that provided solutions to some practical challenges in this library especially his work on [async support](https://github.com/richardanaya/web.rs/blob/master/crates/web/src/executor.rs). Also, to [Greg Johnston](https://github.com/gbj) for [his videos](https://www.youtube.com/@gbjxc/videos) that made working with signals in Rust easy.
