use std::future::Future;

/// dox
pub trait ResponseHandler<Fut, O>:
    'static + FnOnce(reqwest::Result<reqwest::Response>) -> Fut
where
    Fut: BoundedFuture<O>,
{
}
impl<T, Fut, O> ResponseHandler<Fut, O> for T
where
    T: 'static + FnOnce(reqwest::Result<reqwest::Response>) -> Fut,
    Fut: BoundedFuture<O>,
{
}

/// dox
pub trait BoundedFuture<O>: Future<Output = O> {}
impl<T, O> BoundedFuture<O> for T where T: Future<Output = O> {}

/// dox
pub trait DoneHandler<O>: 'static + FnOnce(reqwest::Result<reqwest::Response>) -> O
where
    O: BoundedFuture<()>,
{
}
impl<T, O: BoundedFuture<()>> DoneHandler<O> for T where
    T: 'static + FnOnce(reqwest::Result<reqwest::Response>) -> O
{
}

/// dox
pub trait UiCallBack: 'static + FnOnce() {}
impl<T> UiCallBack for T where T: 'static + FnOnce() {}

/// dox
pub trait ValidReturn: 'static {}
impl<T: 'static> ValidReturn for T {}
