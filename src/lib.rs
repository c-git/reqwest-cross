#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
// dox - used as documentation for duplicate wasm functions (Uncertain if this will cause problems
// but seen this in Reqwest)

//! # reqwest-cross
//!
//! The `reqwest-cross` crate (inspired by [ehttp][ehttp-url]) provides a
//! wrapper around [reqwest][reqwest-url] for ease of use in applications that
//! target BOTH native and wasm and do not want to block in the calling
//! task/thread, for example in a UI task/thread or game loop. This is achieved
//! by using callbacks. This crate provides a few options to choose from and the
//! one that fits best for you depends on what you need. A good way to get an
//! idea what level of abstraction would work for you is by looking at the
//! [examples][#examples]. I would say if you're writing a larger application
//! then [DataState] can abstract away a lot of the boiler plate. In addition I
//! would prefer [fetch_plus]  over [fetch] unless you don't need the UI
//! callback and [fetch_plus] ends up as the one with more boiler plate. If
//! automated retires are desired see [DataStateRetry] which exposes similar
//! methods but with retry built in.
//!
//! NOTE: At least 1 [feature flag](#feature-flags) for
//! native MUST be set to choose which runtime to use. Currently only Tokio is
//! supported but if you want to use another runtime please open an issue on
//! github and we'd be happy to add it. To communicate between the callback and
//! the caller you can use various approaches such as:
//!
//! - The helper type in this crate [DataState] see [examples
//!   folder][examples_folder]
//! - channels  (used in [examples](#examples))
//! - `Arc<Mutex<_>>`
//! - promises and so on.
//!
//! # Examples
//!
//! For examples of how to use this crate see [fetch], [fetch_plus] and the
//! [examples folder][examples_folder] in the repo
//!
//! # Feature Flags
#![doc = document_features::document_features!()]
//!
//! Exactly 1 of the "native-*" flags MUST be enabled to select which runtime to
//! use for native. If one of the other options needs to be used instead of
//! tokio then defaults must be disabled.
//!
//! # Tradeoffs
//!
//! ## Exposing underlying framework that actually sends the requests
//!  The currently selected approach of exposing and using
//! [reqwest::RequestBuilder](https://docs.rs/reqwest/latest/reqwest/struct.RequestBuilder.html) is much more flexible than the fully isolated
//! approach used by [ehttp][ehttp-url] and is generally desirable as it allows
//! for reuse of the same [reqwest::Client][reqwest_client] as recommended by
//! [reqwest][reqwest-url]. However, since [reqwest::Client][reqwest_client] is
//! asynchronous it requires an available runtime. For wasm the spawning of the
//! futures is handled by [wasm-bindgen-futures](https://docs.rs/wasm-bindgen-futures/latest/wasm_bindgen_futures/)
//! but for local which runtime is specified using [feature
//! flags](#feature-flags).
//!
//! # How to run tokio on "secondary" thread
//!
//! If you want to use the main thread for your UI and need to run
//! [tokio][tokio-url] on a "secondary" thread I found this
//! [example](https://github.com/parasyte/egui-tokio-example) helpful. I found it in this
//! [discussion](https://github.com/emilk/egui/discussions/521), which had other suggested
//! examples as well.
//!
//! [reqwest-url]: https://docs.rs/reqwest/latest/reqwest/
//! [ehttp-url]: https://docs.rs/ehttp/0.2.0/ehttp/
//! [tokio-url]: https://docs.rs/tokio/latest/tokio/
//! [reqwest_client]:https://docs.rs/reqwest/latest/reqwest/struct.Client.html
//! [examples_folder]: https://github.com/c-git/reqwest-cross/tree/main/examples

mod data_state;
mod data_state_retry;
mod platform;
mod traits;
#[cfg(feature = "yield_now")]
mod yield_;

pub use data_state::{Awaiting, DataState, DataStateError, ErrorBounds};
pub use data_state_retry::DataStateRetry;
pub use platform::{fetch, fetch_plus, spawn};
pub use traits::{BoundedFuture, DoneHandler, ResponseHandler, UiCallBack, ValidReturn};
#[cfg(feature = "yield_now")]
pub use yield_::yield_now;

// Exported to ensure version used matches
pub use futures::channel::oneshot;
pub use reqwest;
