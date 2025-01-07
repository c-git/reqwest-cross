//! Stores the code specific to wasm compilations

/// dox
pub fn fetch<F, O>(request: reqwest::RequestBuilder, on_done: F)
where
    F: 'static + FnOnce(reqwest::Result<reqwest::Response>) -> O,
    O: futures::Future<Output = ()>,
{
    let future = async move {
        let result = request.send().await;
        on_done(result).await;
    };
    spawn(future);
}

/// dox
pub fn spawn<F>(future: F)
where
    F: futures::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
