//! Stores the code specific to wasm compilations

use reqwest::RequestBuilder;

pub(crate) fn fetch(
    request: RequestBuilder,
    on_done: impl 'static + Send + FnOnce(Result<reqwest::Response, reqwest::Error>),
) {
    wasm_bindgen_futures::spawn_local(async move {
        let result = request.send().await;
        on_done(result)
    });
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use reqwest::{Client, Error, Response};
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    async fn test_fetch() {
        let request = Client::new().get("http://httpbin.org/get");
        let (tx, rx) = channel();

        super::fetch(request, move |result: Result<Response, Error>| {
            tx.send(result.expect("msg").status()).unwrap();
        });

        let status = rx.recv().unwrap();
        assert_eq!(status, 200);
    }
}
