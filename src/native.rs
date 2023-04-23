//! Stores the code specific to native compilations

use reqwest::{Error, RequestBuilder, Response};

pub(crate) fn fetch(
    request: RequestBuilder,
    on_done: impl 'static + Send + FnOnce(Result<Response, Error>),
) {
    // TODO 2: Handle case where feature is not tokio
    tokio::spawn(async move {
        let result = request.send().await;
        on_done(result)
    });
}
