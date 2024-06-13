/// Attempts to provide a yield for executor currently in use.
/// There doesn't appear to be one available as yet for wasm_bindgen_futures so
/// a workaround taken from
/// https://github.com/rustwasm/wasm-bindgen/discussions/3476 is used
pub async fn yield_now() {
    #[cfg(target_arch = "wasm32")]
    sleep_ms(1).await;
    #[cfg(not(target_arch = "wasm32"))]
    tokio::task::yield_now().await;
}

#[cfg(target_arch = "wasm32")]
// Hack to get async sleep on wasm
// Taken from https://github.com/rustwasm/wasm-bindgen/discussions/3476
async fn sleep_ms(millis: i32) {
    let mut cb = |resolve: js_sys::Function, _reject: js_sys::Function| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, millis)
            .expect("Failed to call set_timeout");
    };
    let p = js_sys::Promise::new(&mut cb);
    wasm_bindgen_futures::JsFuture::from(p).await.unwrap();
}
