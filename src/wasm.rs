//! Stores the code specific to wasm compilations

pub fn fetch<F>(request: reqwest::RequestBuilder, on_done: F)
where
    F: 'static + Send + FnOnce(reqwest::Result<reqwest::Response>),
{
    wasm_bindgen_futures::spawn_local(async move {
        let result = request.send().await;
        on_done(result)
    });
}
