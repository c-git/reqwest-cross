//! Provides an easy way for calling functions to reuse bounds of public
//! functions except [spawn][crate::spawn] for which I couldn't foresee the use
//! case. If you have a use case reach out, don't mind adding it if it adds
//! value. All these traits come with automatic impls so they are automatically
//! implemented for any function that meets the bounds functions that are
//! conditionally compiled by target
// Unable to actually include this text for docs.rs because when this module is

// public the traits it includes do not show up as Traits at the top level of
// the crate.

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

/// A function able to be used as a Call Back to notify the UI that the request
/// is ready
pub trait UiCallBack: 'static + Send + FnOnce() {}
impl<T> UiCallBack for T where T: 'static + Send + FnOnce() {}

/// Allowed return types
pub trait ValidReturn: Send + 'static {}
impl<T: Send + 'static> ValidReturn for T {}
