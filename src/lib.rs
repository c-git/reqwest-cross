#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![forbid(unsafe_code)]
#![cfg_attr(test, deny(warnings))]
// dox - used as documentation for duplicate wasm functions (Uncertain if this will cause problems
// but seen this in Reqwest)

//! # reqwest-cross
//!
//! The `reqwest-cross` crate (inspired by [ehttp][ehttp-url]) provides a
//! wrapper around [reqwest][reqwest-url] for ease of use in applications that
//! target BOTH native and wasm and do not want to block in the calling
//! task/thread, for example in a UI task/thread or game loop. This is achieved
//! by using callbacks. NOTE: At least 1 [feature flag](#feature-flags) for
//! native MUST be set to choose which runtime to use. Currently only Tokio is
//! supported but if you want to use another runtime please open an issue on
//! github and we'd be happy to add it. To communicate between the callback and
//! the caller you can use various approaches such as:
//!
//! - channels  (used in [examples](#examples))
//! - `Arc<Mutex<_>>`
//! - promises and so on.
//!
//! # Examples
//!
//! For examples of how to use this crate see [fetch]
//!
//! # Feature Flags
//!
//! Exactly 1 of the "native-*" flags MUST be enabled to select which runtime to
//! use for native. If one of the other options needs to be used instead of
//! tokio then defaults must be disabled.
//!
//! - **default**: Enables the tokio runtime and reqwest's default features. If
//!   disabled do enable requests default features again (or at least tls).
//! - **native-tokio**: Sets [tokio][tokio-url] as the runtime to use for
//!   native. (Default)
//! - reqwest_default: Enables reqwest's default features
//! - **yield_now**: Add a function that can be called to yield to the executor.
//!   This is only needed if you only have one thread and need to release it to
//!   prevent a deadlock because you are waiting on another future (as can be
//!   the case in WASM).
//!
//! A subset of reqwest features have corresponding feature flags on this create
//! to enable them. If you need on that we didn't include please open an issue
//! and let us know and we'll add it. In the mean while you can depend on
//! reqwest directly with the same version as this crate and enable the feature.
//! Because features are additive it will be enabled but note that if the
//! version goes out of sync you're going to have a bad time with being unable
//! to compile.
//!
//! - default-tls
//! - native-tls
//! - rustls-tls
//! - http2
//! - json
//! - cookies
//! - hickory-dns
//! - multipart
//! - socks
//! - stream
//! - brotli
//! - deflate
//! - gzip
//! - zstd
//!
//!
//! # Tradeoffs
//!
//! # Exposing underlying framework that actually sends the requests
//!  The currently selected approach of exposing and using
//! [reqwest::RequestBuilder] is much more flexible than the fully isolated
//! approach used by [ehttp][ehttp-url] and is generally desirable as it allows
//! for reuse of the same [reqwest::Client] as recommended by
//! [reqwest][reqwest-url].However, since [reqwest::Client] is asynchronous it
//! requires an available runtime. For wasm the spawning of the futures is
//! handled by [wasm-bindgen-futures](https://docs.rs/wasm-bindgen-futures/latest/wasm_bindgen_futures/)
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

// TODO 1: test using `document_features` for documenting features

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;
mod wrappers;
#[cfg(feature = "yield_now")]
mod yield_;

pub use wrappers::fetch;

#[cfg(feature = "yield_now")]
pub use yield_::yield_now;

pub use reqwest; // Exported to make it easier to use without a second import and maintain semver
