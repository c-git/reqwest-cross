// Native and WASM require different main functions but after that it should be
// the same. This example demonstrates how this crate can be used with the
// DataState type.

use anyhow::Context;
use reqwest_cross::{fetch_plus, reqwest, Awaiting, DataState};

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
    // Allows for one iteration where we see no progress but next loop should go
    // into first branch
    let mut seen_no_progress = false;

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
            let outcome = state.get(|| {
                let req = client.get("http://httpbin.org/get");
                let response_handler = |resp: reqwest::Result<reqwest::Response>| async {
                    resp.map(|resp| resp.status())
                        .context("Request failed, got an error back")
                };
                let ui_notify = || {
                    println!("Request Completed, this is where you would wake up your UI thread");
                };
                Awaiting(fetch_plus(req, response_handler, ui_notify))
            });
            assert!(!seen_no_progress);
            if outcome.is_unable_to_make_progress() {
                // We should never get into this branch again
                seen_no_progress = true;
            }
            reqwest_cross::yield_now().await;
        }
    }
    println!("Exited loop");
    Ok(())
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {

    #[tokio::test]
    async fn test_name() {
        super::common_code().await.unwrap();
    }
}
