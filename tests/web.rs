use reqwest_cross::fetch;
use wasm_bindgen_test::wasm_bindgen_test;
use wasm_bindgen_test::wasm_bindgen_test_configure;

wasm_bindgen_test_configure!(run_in_browser);
fn main() {
    #[wasm_bindgen_test]
    async fn test_fetch() -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let request = client.get("http://httpbin.org/get");
        let (tx, rx) = futures::channel::oneshot::channel();

        fetch(
            request,
            move |result: Result<reqwest::Response, reqwest::Error>| async {
                tx.send(result.expect("Expecting Response not Error").status())
                    .expect("Receiver should still be available");
            },
        );

        let status = rx.await?; //If we can't block the calling task use try_recv instead
        assert_eq!(status, 200);
        Ok(())
    }
}
