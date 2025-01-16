// Native and WASM require different main functions but after that it should be
// the same. This example demonstrates how this crate can be used with the
// DataState type.

use anyhow::Context;
use reqwest_cross::{fetch_plus, oneshot, reqwest, DataState};

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
        if state.is_none() {
            let client = client.clone();
            let can_make_progress =
                state.start_request(|| make_request(client, "https://httpbin.org/get"));
            assert!(can_make_progress.is_able_to_make_progress());
        }
        if let Some(status_code) = state.poll().present() {
            println!("Response received");
            assert_eq!(status_code, &200);
            break;
        }
        reqwest_cross::yield_now().await;
    }
    println!("Exited loop");
    Ok(())
}

fn make_request(
    client: reqwest::Client,
    url: impl reqwest::IntoUrl,
) -> oneshot::Receiver<anyhow::Result<reqwest::StatusCode>> {
    let req = client.get(url);
    let response_handler = |resp: reqwest::Result<reqwest::Response>| async {
        resp.map(|resp| resp.status())
            .context("Request failed, got an error back")
    };
    let ui_notify = || {
        println!("Request Completed, this is where you would wake up your UI thread.
If using egui version of the functions the associated methods add spinners which will keep the loop going so no wake up is needed.
Passing an empty closure would suffice.");
    };
    fetch_plus(req, response_handler, ui_notify)
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {

    #[tokio::test]
    async fn test_name() {
        super::common_code().await.unwrap();
    }
}
