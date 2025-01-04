//! Stores the code specific to native compilations
// but the comments are for both because these are the ones that show on docs.rs

#[cfg(not(feature = "native-tokio"))]
compile_error!("Must chose a native runtime by enabling a feature flag. Right now only tokio is supported. If you have a different runtime that you want please create an issue on github.");

/// Performs a HTTP requests and calls the given callback when done with the
/// result of the request. This is a more flexible API but requires more
/// boilerplate, see [fetch_plus] which wraps a lot more of the boilerplate
/// especially if you need a "wake_up" function.  NB: Needs to use a callback to
/// prevent blocking on the thread that initiates the fetch. Note: Instead of
/// calling get like in the example you can use post, put, etc. (See
/// [reqwest::Client]). Also see the examples
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

/// Spawns a future on the underlying runtime in a cross platform way
#[cfg(feature = "native-tokio")]
pub fn spawn<F>(future: F)
where
    F: 'static + Send + futures::Future,
    F::Output: Send + 'static,
{
    tokio::spawn(future);
}
