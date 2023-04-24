use futures::channel::oneshot::channel;
use reqwest::{Client, Error, Response};
use reqwest_cross::fetch;
use wasm_bindgen_test::wasm_bindgen_test;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);
fn main() {
    #[wasm_bindgen_test]
    async fn test_fetch() -> Result<(), Box<dyn std::error::Error>> {
        let request = Client::new().get("http://httpbin.org/get");
        let (tx, rx) = channel();

        fetch(request, move |result: Result<Response, Error>| {
            tx.send(result.expect("Expecting Response not Error").status())
                .expect("Receiver should still be available");
        });

        let status = rx.await?; //In actual use case code to prevent blocking use try_recv instead
        assert_eq!(status, 200);
        Ok(())
    }
}
