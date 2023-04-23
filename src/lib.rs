#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]

//! # reqwest-cross
//!
//! The `reqwest-cross` crate provides a wrapper around [reqwest][reqwest-url] for ease of use in applications that target BOTH native and wasm.
//! This crate is inspired by [ehttp](https://docs.rs/ehttp/0.2.0/ehttp/) but uses [reqwest][reqwest-url] instead.
//!
//! The currently selected approach of exposing and using [reqwest::RequestBuilder] is much more flexible and generally desirable
//! as it allows for reuse of the same [reqwest::Client] as recommended by [reqwest][reqwest-url]. However, since it is asynchronous it requires
//! an available runtime. For wasm the futures are handled by [wasm-bindgen-futures](https://docs.rs/wasm-bindgen-futures/latest/wasm_bindgen_futures/)
//! but for local which runtime is specified using feature flags. If the one you want is not listed create an [issue](https://github.com/c-git/reqwest-cross/issues)
//! and I'll attempt to add it.
//!
//! If you want to use the main thread for your UI and need to run [tokio](https://docs.rs/tokio/latest/tokio/) on a "secondary" thread
//! I found this [example](https://github.com/parasyte/egui-tokio-example) helpful. I found it in this
//! [discussion](https://github.com/emilk/egui/discussions/521), which had other suggested examples as well.
//!
//! [reqwest-url]: https://docs.rs/reqwest/latest/reqwest/

// TODO 3: Add an example to the documentation

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm; // TODO 2: Implement version for web
mod wrappers;

#[cfg(target_arch = "wasm32")]
pub use web::{fetch_async, spawn_future};

pub use wrappers::fetch;
