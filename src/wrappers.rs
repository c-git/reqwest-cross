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
    use std::sync::{Arc, Mutex};

    use reqwest::Client;

    use super::*;

    #[cfg(not(target_arch = "wasm32"))] // TODO 2: Also needs to be behind the tokio feature flag
    #[tokio::test]
    async fn native_get() {
        let request = Client::new().get("http://httpbin.org/get");
        let response: Arc<Mutex<Option<Result<Response, Error>>>> = Arc::new(Mutex::new(None));
        let closure_copy = Arc::clone(&response);
        fetch(request, move |result: Result<Response, Error>| {
            let mut value = closure_copy.lock().unwrap();
            *value = Some(result);
        });
        let status = loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            let option = response.lock().expect("Get mutex value");
            if let Some(result) = option.as_ref() {
                let status = result
                    .as_ref()
                    .expect("Expecting Response not Error")
                    .status();
                break status;
            }
        };

        assert_eq!(status, 200);
    }

    // TODO 3: Add tests for post
    // TODO 3: Add tests for patch
    // TODO 3: Add tests for delete
    // TODO 3: Add tests for put
}
