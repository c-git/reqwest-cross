//! Stores the wrapper functions that can be called from either native or wasm code

use reqwest::{Error, RequestBuilder, Response};

/// Performs a HTTP requests and calls the given callback when done. NB: Needs to use a callback to prevent blocking on the thread that initiates the fetch.
///
/// # Tokio example
/// ```rust
///# use reqwest::{Client, Error, Response};
///# use tokio::sync::oneshot;
///# use reqwest_cross::fetch;
///
///# #[cfg(all(not(target_arch = "wasm32"),feature = "native-tokio"))]
///# #[tokio::main(flavor = "current_thread")]
///# async fn main() {
///  let request = Client::new().get("http://httpbin.org/get");
///  let (tx, rx) = oneshot::channel();
///
///  fetch(request, move |result: Result<Response, Error>| {
///      tx.send(result.expect("Expecting Response not Error").status()).unwrap();
///  });
///
///  let status = rx
///      .await
///      .unwrap();
///  assert_eq!(status, 200);
///# }
///
///# #[cfg(target_arch = "wasm32")]
///# fn main(){}
/// ```
pub fn fetch(
    request: RequestBuilder,
    on_done: impl 'static + Send + FnOnce(Result<Response, Error>),
) {
    #[cfg(not(target_arch = "wasm32"))]
    crate::native::fetch(request, Box::new(on_done));

    #[cfg(target_arch = "wasm32")]
    crate::wasm::fetch(request, Box::new(on_done));
}

#[cfg(test)]
mod tests {

    // TODO 3: Add tests for post (create as examples instead of tests)
    // TODO 3: Add tests for patch
    // TODO 3: Add tests for delete
    // TODO 3: Add tests for put
}
