// Native and WASM require different main functions but after that it should be the same
//
// I haven't tried running this code in wasm as in my use case I use egui and don't deal with the WASM directly
// but see example here https://github.com/seanmonstar/reqwest/tree/master/examples/wasm_github_fetch if you want to run wasm directly

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

async fn common_code() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let request = client.get("http://httpbin.org/get");
    let (tx, rx) = futures::channel::oneshot::channel();

    fetch(
        request,
        move |result: Result<reqwest::Response, reqwest::Error>| {
            tx.send(result.expect("Expecting Response not Error"))
                .expect("Receiver should still be available");
        },
    );

    // Note the next call block this execution path (task / thread) see loop example for alternative
    let status = rx.await?.status();
    assert_eq!(status, 200);
    Ok(())
}
