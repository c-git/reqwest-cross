//! Stores the wrapper functions that can be called from either native or wasm
//! code

use std::{fmt::Debug, future::Future};
use tracing::error;

#[cfg(not(target_arch = "wasm32"))]
/// Performs a HTTP requests and calls the given callback when done. NB: Needs
/// to use a callback to prevent blocking on the thread that initiates the
/// fetch. Note: Instead of calling get like in the example you can use post,
/// put, etc. (See [reqwest::Client]). Also see the examples
/// [folder](https://github.com/c-git/reqwest-cross/tree/main/examples)
/// for more complete examples.
///
/// # Tokio example
/// ```rust
/// # use reqwest_cross::fetch;
///
/// # #[cfg(all(not(target_arch = "wasm32"),feature = "native-tokio"))]
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///  let client = reqwest::Client::new();
///  let request = client.get("http://httpbin.org/get");
///  let (tx, rx) = futures::channel::oneshot::channel();
///
///  fetch(request, move |result: Result<reqwest::Response, reqwest::Error>| async {
///      tx.send(result.expect("Expecting Response not Error").status())
///                .expect("Receiver should still be available");
///  });
///
///  let status = rx.await?; //In actual use case code to prevent blocking use try_recv instead
///  assert_eq!(status, 200);
/// # Ok(())
/// # }
///
/// # #[cfg(target_arch = "wasm32")]
/// # fn main(){}
/// ```
pub fn fetch<F, O>(request: reqwest::RequestBuilder, on_done: F)
where
    F: 'static + Send + FnOnce(reqwest::Result<reqwest::Response>) -> O,
    O: futures::Future<Output = ()> + Send,
{
    let future = async move {
        let result = request.send().await;
        on_done(result).await;
    };
    spawn(future);
}

/// dox
#[cfg(target_arch = "wasm32")]
pub fn fetch<F, O>(request: reqwest::RequestBuilder, on_done: F)
where
    F: 'static + FnOnce(reqwest::Result<reqwest::Response>) -> O,
    O: futures::Future<Output = ()>,
{
    let future = async move {
        let result = request.send().await;
        on_done(result).await;
    };
    spawn(future);
}

#[cfg(not(target_arch = "wasm32"))]
/// Spawns a future on the underlying runtime in a cross platform way
pub fn spawn<F>(future: F)
where
    F: futures::Future<Output = ()> + 'static + Send,
{
    crate::native::spawn(future);
}

#[cfg(target_arch = "wasm32")]
/// dox
pub fn spawn<F>(future: F)
where
    F: futures::Future<Output = ()> + 'static,
{
    crate::wasm::spawn(future);
}

/// Wraps the call to fetch with the surrounding boilerplate
pub fn fetch_plus<FResponseHandler, FNotify, Fut, Ret>(
    req: reqwest::RequestBuilder,
    response_handler: FResponseHandler,
    ui_notify: FNotify,
) -> crate::oneshot::Receiver<anyhow::Result<Ret>>
where
    FResponseHandler: FnOnce(reqwest::Result<reqwest::Response>) -> Fut + Send + 'static,
    Fut: Future<Output = anyhow::Result<Ret>> + Send,
    Ret: Send + 'static,
    Fut::Output: Debug,
    FNotify: FnOnce() + Send + 'static,
{
    let (tx, rx) = crate::oneshot::channel();
    let on_done = move |resp: reqwest::Result<reqwest::Response>| async {
        let output = response_handler(resp).await;
        match tx.send(output) {
            Ok(()) => {}
            Err(handler_output) => error!(?handler_output, "failed to send output from handler"),
        };
        ui_notify();
    };
    fetch(req, on_done);
    rx
}
