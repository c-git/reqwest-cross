//! Stores the wrapper functions that can be called from either native or wasm code

use reqwest::{Error, RequestBuilder, Response};

/// Performs a HTTP requests and calls the given callback when done.
///
/// # Tokio example
/// ```rust
///# use reqwest::{Client, Error, RequestBuilder, Response};
///# use std::sync::mpsc::channel;
///# use reqwest_cross::fetch;
///
///# #[cfg(not(target_arch = "wasm32"))] // TODO 2: Also needs to be behind the tokio feature flag
///# #[tokio::main]
///# async fn main(){
///  let request = Client::new().get("http://httpbin.org/get");
///  let (tx, rx) = channel();
///  fetch(request, move |result: Result<Response, Error>| {
///      tx.send(result).unwrap();
///  });
///  let status = loop {
///      // Sleep too prevent locking up the thread and not awaiting
///      tokio::time::sleep(std::time::Duration::from_millis(100)).await;
///      if let Ok(val) = rx.try_recv() {
///          break val.unwrap().status();
///      }
///  };
///
///  assert_eq!(status, 200);
///# }
///
///# #[cfg(target_arch = "wasm32")] // TODO 2: Also needs to be behind the tokio feature flag
///# fn main(){}
/// ```
pub fn fetch(
    request: RequestBuilder,
    on_done: impl 'static + Send + FnOnce(Result<Response, Error>),
) {
    #[cfg(not(target_arch = "wasm32"))]
    crate::native::fetch(request, Box::new(on_done));

    #[cfg(target_arch = "wasm32")]
    crate::web::fetch(request, Box::new(on_done));
}

#[cfg(test)]
mod tests {
    // TODO 3: Add tests for post (create as examples instead of tests)
    // TODO 3: Add tests for patch
    // TODO 3: Add tests for delete
    // TODO 3: Add tests for put
}
