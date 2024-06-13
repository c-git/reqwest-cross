use reqwest_cross::fetch;

#[cfg(all(not(target_arch = "wasm32"), feature = "native-tokio"))]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    common_code().await
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
fn main() {
    common_code().await.unwrap();
}

enum State {
    Startup,
    AwaitingResponse(futures::channel::oneshot::Receiver<reqwest::Response>),
    Done,
}

async fn common_code() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let mut state = State::Startup;

    // This loop would normally be a game loop, or the executor of an immediate mode GUI.
    loop {
        match &mut state {
            State::Startup => {
                // Send request
                let request = client.get("http://httpbin.org/get");
                let (tx, rx) = futures::channel::oneshot::channel();
                fetch(
                    request,
                    move |result: Result<reqwest::Response, reqwest::Error>| {
                        // You can also return the result instead of using expect but made the example more complicated to read
                        tx.send(result.expect("Expecting Response not Error"))
                            .expect("Receiver should still be available");
                    },
                );
                println!("Request sent");
                state = State::AwaitingResponse(rx);
            }
            State::AwaitingResponse(rx) => {
                // Check if response is ready
                match rx.try_recv() {
                    Ok(option) => {
                        if let Some(response) = option {
                            let status = response.status();
                            println!("Response received");
                            assert_eq!(status, 200);
                            state = State::Done;
                        } else {
                            // Still waiting
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
