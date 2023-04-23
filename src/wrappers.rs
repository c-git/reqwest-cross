//! Stores the wrapper functions that can be called from either native or wasm code

use reqwest::{Error, RequestBuilder, Response};

/// Performs a HTTP requests and calls the given callback when done.
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
    use reqwest::Client;
    use std::sync::mpsc::channel;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))] // TODO 2: Also needs to be behind the tokio feature flag
    #[tokio::test]
    async fn native_get() {
        let request = Client::new().get("http://httpbin.org/get");
        let (tx, rx) = channel();
        fetch(request, move |result: Result<Response, Error>| {
            tx.send(result).unwrap();
        });
        let status = loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if let Ok(val) = rx.try_recv() {
                break val.unwrap().status();
            }
        };

        assert_eq!(status, 200);
    }

    // TODO 3: Add tests for post
    // TODO 3: Add tests for patch
    // TODO 3: Add tests for delete
    // TODO 3: Add tests for put
}
