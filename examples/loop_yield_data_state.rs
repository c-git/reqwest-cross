// Native and WASM require different main functions but after that it should be
// the same. This example demonstrates how this crate can be used with the DataState type.

use anyhow::Context;
use reqwest::{Method, RequestBuilder, StatusCode};
use reqwest_cross::{fetch, reqwest, Awaiting, DataState};

#[cfg(all(not(target_arch = "wasm32"), feature = "native-tokio"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    common_code().await
}

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
#[cfg(target_arch = "wasm32")]
fn main() {
    #[wasm_bindgen_test::wasm_bindgen_test]
    async fn do_fetch() -> Result<(), Box<dyn std::error::Error>> {
        common_code().await
    }
}

async fn common_code() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut state = DataState::None;

    println!("Starting loop");

    // This loop would normally be a game loop, or the executor of an immediate mode
    // GUI.
    loop {
        if let DataState::Present(status_code) = state.as_ref() {
            println!("Response received");
            assert_eq!(status_code, &200);
            break;
        } else {
            state.get(|| {
                let req = client.request(Method::GET, "http://httpbin.org/get");
                Awaiting(send_request_give_back_status(req, || {
                    println!("Request Completed, this is where you would wake up your UI thread");
                }))
            });
            reqwest_cross::yield_now().await;
        }
    }
    println!("Exited loop");
    Ok(())
}

fn send_request_give_back_status<F>(
    req: RequestBuilder,
    ui_notify: F,
) -> reqwest_cross::oneshot::Receiver<anyhow::Result<StatusCode>>
where
    F: FnOnce() + Send + 'static,
{
    let (tx, rx) = reqwest_cross::oneshot::channel();
    let on_done = move |resp: reqwest::Result<reqwest::Response>| async {
        let status_code = resp
            .map(|resp| resp.status())
            .context("Request failed, got an error back");
        tx.send(status_code).expect("failed to send oneshot msg");
        ui_notify();
    };
    fetch(req, on_done);
    println!("Request sent");
    rx
}

#[cfg(all(test, not(target_arch = "wasm32")))]

mod tests {

    #[tokio::test]
    async fn test_name() {
        super::common_code().await.unwrap();
    }
}
