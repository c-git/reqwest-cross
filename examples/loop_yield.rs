// Native and WASM require different main functions but after that it should be
// the same. Uses yield but yield isn't available yet for wasm_bindgen_futures so
// uses a workaround found (poll-promise might be better)

use reqwest_cross::{fetch, reqwest};

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

enum State {
    Startup,
    AwaitingResponse(futures::channel::oneshot::Receiver<reqwest::StatusCode>),
    Done,
}

async fn common_code() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut state = State::Startup;

    println!("Starting loop");

    // This loop would normally be a game loop, or the executor of an immediate mode
    // GUI.
    loop {
        match state {
            State::Startup => {
                // Send request
                let request = client.get("http://httpbin.org/get");
                let (tx, rx) = futures::channel::oneshot::channel();
                fetch(
                    request,
                    move |result: Result<reqwest::Response, reqwest::Error>| async {
                        tx.send(result.expect("Expecting Response not Error").status())
                            .expect("Receiver should still be available");
                    },
                );
                println!("Request sent");
                state = State::AwaitingResponse(rx);
            }
            State::AwaitingResponse(mut rx) => {
                // Check if response is ready
                match rx.try_recv() {
                    Ok(option) => {
                        if let Some(status) = option {
                            println!("Response received");
                            assert_eq!(status, 200);
                            state = State::Done;
                        } else {
                            // Still waiting
                            state = State::AwaitingResponse(rx);
                            reqwest_cross::yield_now().await;
                        }
                    }
                    Err(e) => {
                        eprintln!("Canceled");
                        return Err(Box::new(e));
                    }
                }
            }
            State::Done => {
                // All done exit now
                println!("Completed exiting");
                return Ok(());
            }
        }
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]

mod tests {

    #[tokio::test]
    async fn test_name() {
        super::common_code().await.unwrap();
    }
}
