//! Stores the wrapper functions that can be called from either native or wasm
//! code

use tracing::error;
#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

// Using * imports to bring them up to this level
use crate::{BoundedFuture, ResponseHandler, UiCallBack, ValidReturn};
#[cfg(not(target_arch = "wasm32"))]
pub use native::*;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;

/// Wraps the call to [fetch] with the surrounding boilerplate.
///
/// # Example
/// ```rust
/// # use reqwest_cross::fetch_plus;
/// #
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = reqwest::Client::new();
///     let request = client.get("http://httpbin.org/get");
///     let handler = |result: Result<reqwest::Response, reqwest::Error>| async {
///         result.expect("Expecting Response not Error").status()
///     };
///     let rx = fetch_plus(request, handler, || {});
///     let status = rx.await?; //In actual use case code to prevent blocking use try_recv instead
///     assert_eq!(status, 200);
/// #    Ok(())
/// # }
///
/// # #[cfg(target_arch = "wasm32")]
/// # fn main(){}
/// ```
pub fn fetch_plus<FResponseHandler, FNotify, Fut, Ret>(
    req: reqwest::RequestBuilder,
    response_handler: FResponseHandler,
    ui_notify: FNotify,
) -> crate::oneshot::Receiver<Ret>
where
    FResponseHandler: ResponseHandler<Fut, Ret>,
    Fut: BoundedFuture<Ret>,
    Ret: ValidReturn,
    FNotify: UiCallBack,
{
    let (tx, rx) = crate::oneshot::channel();
    let on_done = move |resp: reqwest::Result<reqwest::Response>| async {
        let output = response_handler(resp).await;
        match tx.send(output) {
            Ok(()) => {}
            Err(_output) => error!("failed to send output from handler"),
        };
        ui_notify();
    };
    fetch(req, on_done);
    rx
}
