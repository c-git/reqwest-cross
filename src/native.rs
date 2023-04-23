//! Stores the code specific to native compilations

#[cfg(not(feature = "native-tokio"))]
compile_error!("Must chose a native runtime by enabling a feature flag. Right now only tokio is supported. If you have a different runtime that you want please create an issue on github.");

#[cfg(feature = "native-tokio")]
pub(crate) fn fetch(
    request: reqwest::RequestBuilder,
    on_done: impl 'static + Send + FnOnce(Result<reqwest::Response, reqwest::Error>),
) {
    // TODO 2: Handle case where feature is not tokio
    tokio::spawn(async move {
        let result = request.send().await;
        on_done(result)
    });
}
