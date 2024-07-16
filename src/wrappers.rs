//! Stores the wrapper functions that can be called from either native or wasm
//! code

use std::fmt::Debug;

use futures::Future;

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
///  fetch(request, move |result: Result<reqwest::Response, reqwest::Error>| {
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
pub fn fetch<F>(request: reqwest::RequestBuilder, on_done: F)
where
    F: 'static + Send + FnOnce(reqwest::Result<reqwest::Response>),
{
    let future = async move {
        let result = request.send().await;
        on_done(result)
    };
    spawn(future);
}

/// Spawns a future on the underlying runtime in a cross platform way
pub fn spawn<F>(future: F)
where
    F: futures::Future<Output = ()> + 'static + Send,
{
    #[cfg(not(target_arch = "wasm32"))]
    crate::native::spawn(future);

    #[cfg(target_arch = "wasm32")]
    crate::wasm::spawn(future);
}

/// Sometimes needed in callbacks to call async code from the sync context
/// Provides a cross platform compatible way to run an async function in a
/// blocking fashion, but without actually blocking because then the function
/// cannot complete and results in a deadlock.
///
/// # Warning
/// Until a better solution can be found this will result in a busy loop so use
/// sparingly.
pub fn wait_for<F>(future: F) -> F::Output
where
    F: Future + Send + 'static,
    F::Output: Debug + Send,
{
    let (tx, mut rx) = futures::channel::oneshot::channel();
    spawn(async move {
        tx.send(future.await)
            .expect("failed to send but expected rx to still be available");
    });
    loop {
        // TODO 4: Is there a better way to do this than a busy loop?
        if let Some(x) = rx
            .try_recv()
            .expect("failed to receive. maybe sender panicked")
        {
            return x;
        }
    }
}

// TODO 3: Test link in documentation after pushing to main
