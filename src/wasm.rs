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
