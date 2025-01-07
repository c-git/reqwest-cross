use std::future::Future;

/// An async func that accepts a [reqwest::Response]
/// and returns a generic value
pub trait ResponseHandler<Fut, O>:
    Send + 'static + FnOnce(reqwest::Result<reqwest::Response>) -> Fut
where
    Fut: BoundedFuture<O>,
{
}
impl<T, Fut, O> ResponseHandler<Fut, O> for T
where
    T: Send + 'static + FnOnce(reqwest::Result<reqwest::Response>) -> Fut,
    Fut: BoundedFuture<O>,
{
}

/// A function that receives the [reqwest::Response]
/// and returns it to the application via some means (See examples for way it
/// can be done)
pub trait DoneHandler<O>: 'static + Send + FnOnce(reqwest::Result<reqwest::Response>) -> O
where
    O: BoundedFuture<()>,
{
}
impl<T, O: BoundedFuture<()>> DoneHandler<O> for T where
    T: 'static + Send + FnOnce(reqwest::Result<reqwest::Response>) -> O
{
}

/// A future with the required bounds for the platform
pub trait BoundedFuture<O>: Future<Output = O> + Send {}
impl<T, O> BoundedFuture<O> for T where T: Future<Output = O> + Send {}

/// A function able to be used as a Call Back to notify the UI that the request
/// is ready
pub trait UiCallBack: 'static + Send + FnOnce() {}
impl<T> UiCallBack for T where T: 'static + Send + FnOnce() {}

/// Allowed return types
pub trait ValidReturn: Send + 'static {}
impl<T: Send + 'static> ValidReturn for T {}
