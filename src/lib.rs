#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(test, deny(warnings))]

//! # reqwest-cross
//!
//! The `reqwest-cross` crate (inspired by [ehttp][ehttp-url]) provides a wrapper around [reqwest][reqwest-url] for ease of use in applications that target
//! BOTH native and wasm and do not want to block in the calling task/thread, for example in a UI task/thread. This is achieved by using callbacks and
//! while not ideal a better solution hasn't been found yet. NOTE: At least 1 [feature flag](#feature-flags) MUST be set to choose which runtime to use for
//! native. To communicate between the callback and the caller you can use various approaches such as channels (used in [examples](#examples)), Arc<Mutex<_>>, promises
//! and so on. [egui](https://docs.rs/egui/latest/egui/) has an example of how to handle the callback (using promises and their repaint request to wake the UI)
//! [here](https://github.com/emilk/egui/tree/master/examples/download_image). Their example uses [ehttp][ehttp-url] instead of `reqwest-cross` but the
//! callback handling can be done the same way. Only real difference in client code is a [reqwest::Client], which requires a runtime, is passed instead of a
//! [ehttp::Request](https://docs.rs/ehttp/latest/ehttp/struct.Request.html).
//!
//! # Examples
//!
//! For examples of how to use this crate see [fetch] and the [tests](https://github.com/c-git/reqwest-cross/tree/main/tests).
//!
//! # Feature Flags
//!
//! Exactly 1 of the "native-*" flags MUST be enabled to select which runtime to use for native.
//!
//! - **native-tokio**: Sets [tokio][tokio-url] as the runtime to use for native. NOTE: Right now this feature is REQUIRED as no other
//! options are provided for native right now. (But not set as default to not break semver if one is added later)
//!
//! # Tradeoffs
//!
//! **Exposing underlying framework that actually sends the requests**: The currently selected approach of exposing and using [reqwest::RequestBuilder] is
//! much more flexible than the fully isolated approach used by [ehttp][ehttp-url] and is generally desirable as it allows for reuse of the same
//! [reqwest::Client] as recommended by [reqwest][reqwest-url].However, since [reqwest::Client] is asynchronous it requires an available runtime. For wasm
//! the spawning of the futures is handled by [wasm-bindgen-futures](https://docs.rs/wasm-bindgen-futures/latest/wasm_bindgen_futures/)
//! but for local which runtime is specified using [feature flags](#feature-flags). If the one you want is not listed please create an
//! [issue](https://github.com/c-git/reqwest-cross/issues) and I'll attempt to add it.
//!
//! # How to run tokio on "secondary" thread
//!
//! If you want to use the main thread for your UI and need to run [tokio][tokio-url] on a "secondary" thread
//! I found this [example](https://github.com/parasyte/egui-tokio-example) helpful. I found it in this
//! [discussion](https://github.com/emilk/egui/discussions/521), which had other suggested examples as well.
//!
//! [reqwest-url]: https://docs.rs/reqwest/latest/reqwest/
//! [ehttp-url]: https://docs.rs/ehttp/0.2.0/ehttp/
//! [tokio-url]: https://docs.rs/tokio/latest/tokio/

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;
mod wrappers;

pub use wrappers::fetch;
