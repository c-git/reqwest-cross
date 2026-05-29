//! Stores the wrapper functions that can be called from either native or wasm
//! code (Most of the platform specific code was moved to the main-loop-async
//! crate)

use main_loop_async::spawn;
use tracing::error;

// Using * imports to bring them up to this level
use crate::{BoundedFuture, DoneHandler, ResponseHandler, UiCallBack, ValidReturn};

/// Wraps the call to [fetch] with the surrounding boilerplate.
///
/// # Example
/// ```rust,ignore-wasm32
/// # use reqwest_cross::fetch_plus;
/// #
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let client = reqwest::Client::new();
///     let request = client.get("https://httpbin.org/get");
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
#[must_use = "receiver is needed to get the response"]
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

/// Performs a HTTP requests and calls the given callback when done with the
/// result of the request. This is a more flexible API but requires more
/// boilerplate, see [fetch_plus][crate::fetch_plus] which wraps a lot more of
/// the boilerplate especially if you need a "wake_up" function.  NB: Needs to
/// use a callback to prevent blocking on the thread that initiates the fetch.
/// Note: Instead of calling get like in the example you can use post, put, etc.
/// (See [reqwest::Client]). Also see the examples
/// [folder](https://github.com/c-git/reqwest-cross/tree/main/examples)
/// for more complete examples.
///
/// # Example
/// ```rust
/// # use reqwest_cross::fetch;
///
/// # #[cfg(all(not(target_arch = "wasm32"),feature = "native-tokio"))]
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
///  let client = reqwest::Client::new();
///  let request = client.get("https://httpbin.org/get");
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
    F: DoneHandler<O>,
    O: BoundedFuture<()>,
{
    let future = async move {
        let result = request.send().await;
        on_done(result).await;
    };
    spawn(future);
}
