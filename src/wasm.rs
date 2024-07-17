//! Stores the code specific to wasm compilations

pub fn spawn<F>(future: F)
where
    F: futures::Future<Output = ()> + 'static,
{
    wasm_bindgen_futures::spawn_local(future);
}
