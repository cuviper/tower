use std::{fmt, marker::PhantomData};

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower_service::Service;

/// A [`Future`] that yields the service when it is ready to accept a request.
///
/// [`ReadyOneshot`] values are produced by [`ServiceExt::ready_oneshot`].
///
/// [`ServiceExt::ready_oneshot`]: crate::util::ServiceExt::ready_oneshot
pub struct ReadyOneshot<T, Request> {
    inner: Option<T>,
    _p: PhantomData<fn() -> Request>,
}

// Safety: This is safe because `Services`'s are always `Unpin`.
impl<T, Request> Unpin for ReadyOneshot<T, Request> {}

impl<'a, T, Request> ReadyOneshot<T, Request>
where
    T: Service<'a, Request>,
{
    #[allow(missing_docs)]
    pub fn new(service: T) -> Self {
        Self {
            inner: Some(service),
            _p: PhantomData,
        }
    }
}

impl<T, Call, Request, Error> Future for ReadyOneshot<T, Request>
where
    T: for<'a> Service<'a, Request, Call = Call, Error = Error>,
{
    type Output = Result<Call, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.inner
            .as_mut()
            .expect("poll after Poll::Ready")
            .poll_ready(cx)
    }
}

impl<T, Request> fmt::Debug for ReadyOneshot<T, Request>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ReadyOneshot")
            .field("inner", &self.inner)
            .finish()
    }
}

/// A future that yields a mutable reference to the service when it is ready to accept a request.
///
/// [`Ready`] values are produced by [`ServiceExt::ready`].
///
/// [`ServiceExt::ready`]: crate::util::ServiceExt::ready
pub struct Ready<'a, T, Request>(ReadyOneshot<&'a mut T, Request>);

// Safety: This is safe for the same reason that the impl for ReadyOneshot is safe.
impl<'a, T, Request> Unpin for Ready<'a, T, Request> {}

impl<'a, T, Request> Ready<'a, T, Request>
where
    T: Service<'a, Request>,
{
    #[allow(missing_docs)]
    pub fn new(service: &'a mut T) -> Self {
        Self(ReadyOneshot::new(service))
    }
}

impl<'a, T, Request, Call, Error> Future for Ready<'a, T, Request>
where
    T: Service<'a, Request>,
    ReadyOneshot<&'a mut T, Request>: Future<Output = Result<Call, Error>>,
{
    type Output = Result<Call, Error>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.0).poll(cx)
    }
}

impl<'a, T, Request> fmt::Debug for Ready<'a, T, Request>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Ready").field(&self.0).finish()
    }
}
