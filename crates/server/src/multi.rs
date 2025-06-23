use std::{
    fmt::{Debug, Display},
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    task::{Context, Poll},
};

use pin_project::pin_project;
use smallvec::SmallVec;
use tokio::{pin, task::JoinSet};
use tower::Service;

/// A service that wraps multiple services of the same type and dispatches
/// requests to all of them concurrently, returning a future that acts as a
/// `JoinSet`.
///
/// This service polls all inner services until they are ready. If any service
/// returns a successful response, it returns the first one. If all services
/// return errors, it collects and returns all errors.
///
/// # Type Parameters
/// - `S`: The inner service type.
/// - `N`: The number of services to manage.
#[derive(Debug, Clone)]
pub struct MultiService<S, const N: usize> {
    inner: [S; N],
}

/// A service that wraps multiple services of the same type and dispatches
/// requests to all of them concurrently.
///
/// After the first service returns a successful response, the remaining
/// services are left to run in the background. This is useful for "fire and
/// forget" scenarios where you only need one success to proceed, but want the
/// others to complete without waiting for them. This service does not
/// provide a `JoinSet`-like mechanism to wait for all services.
///
/// # Type Parameters
/// - `S`: The inner service type.
/// - `N`: The number of services to manage.
#[derive(Debug, Clone)]
pub struct MultiServiceRace<S, const N: usize> {
    inner: [S; N],
}

/// A service that wraps multiple services of the same type and dispatches
/// requests to all of them concurrently.
///
/// This service is similar to `MultiServiceRace`, but with a key difference in
/// error handling. If all services return errors, it will return only one of
/// the errors instead of a collection of all errors.
///
/// # Type Parameters
/// - `S`: The inner service type.
/// - `N`: The number of services to manage.
#[derive(Debug, Clone)]
pub struct MultiServiceRaceAnyError<S, const N: usize> {
    inner: [S; N],
}

impl<S, const N: usize> MultiService<S, N> {
    /// Creates a new `MultiService` from an array of services.
    ///
    /// # Arguments
    /// - `inner`: An array of services to wrap.
    ///
    /// # Returns
    /// A new `MultiService` instance.
    #[must_use]
    pub const fn from_array(inner: [S; N]) -> Self {
        Self { inner }
    }

    #[must_use]
    pub fn race(self) -> MultiServiceRace<S, N> {
        MultiServiceRace::from_array(self.inner)
    }

    /// # Panics
    /// If `N` is zero, this method will panic.
    #[must_use]
    pub fn race_any_error(self) -> MultiServiceRaceAnyError<S, N> {
        MultiServiceRaceAnyError::from_array(self.inner)
    }
}

impl<S, const N: usize> MultiServiceRace<S, N> {
    /// Creates a new `MultiServiceIgnoreJoinSet` from an array of services.
    ///
    /// # Arguments
    /// - `inner`: An array of services.
    ///
    /// # Returns
    /// A new `MultiServiceRace` instance.
    #[must_use]
    pub const fn from_array(inner: [S; N]) -> Self {
        Self { inner }
    }
}

impl<S, const N: usize> MultiServiceRaceAnyError<S, N> {
    /// Creates a new `MultiServiceRaceAnyError` from an array of services.
    ///
    /// # Arguments
    /// - `inner`: An array of services to wrap.
    ///
    /// # Returns
    /// A new `MultiServiceRaceAnyError` instance.
    ///
    /// # Panics
    /// If `N` is zero, this method will panic.
    #[must_use]
    pub const fn from_array(inner: [S; N]) -> Self {
        assert!(
            N > 0,
            "Cannot create a MultiFutureIgnoreMultiError with zero futures"
        );
        Self { inner }
    }
}

/// A future that polls multiple futures concurrently and resolves when any of
/// them completes successfully.
///
/// This struct polls all provided futures until they complete. If any future
/// resolves successfully (`Ok(Ret)`), it returns the first successful response.
/// If all futures resolve with errors (`Err(Error)`), it collects and returns
/// all errors.
///
/// # Type Parameters
/// - `Fut`: The future type to be polled.
/// - `Ret`: The successful return type of the future.
/// - `Error`: The error type of the future.
/// - `N`: The number of futures to manage.
#[pin_project]
pub struct MultiFutureJoin<Fut, Ret, Error, const N: usize>
where
    Fut: Future<Output = Result<Ret, Error>>,
{
    #[pin]
    futures:   MaybeUninit<[Fut; N]>,
    all_error: SmallVec<[Error; N]>,
    ready_map: [bool; N],
    ok:        Option<Ret>,
}

#[pin_project]
pub struct MultiFutureRace<Fut, Ret, Error, const N: usize>
where
    Fut: Future<Output = Result<Ret, Error>>,
{
    #[pin]
    futures:   MaybeUninit<[Fut; N]>,
    all_error: SmallVec<[Error; N]>,
    ready_map: [bool; N],
    ok:        Option<Ret>,
}

#[pin_project]
pub struct MultiFutureRaceAnyError<Fut, Ret, Error, const N: usize>
where
    Fut: Future<Output = Result<Ret, Error>>,
{
    #[pin]
    futures:    MaybeUninit<[Fut; N]>,
    last_error: Option<Error>,
    ready_map:  [bool; N],
    ok:         Option<Ret>,
}

impl<Fut, Ret, Error, const N: usize> MultiFutureJoin<Fut, Ret, Error, N>
where
    Fut: Future<Output = Result<Ret, Error>>,
{
    /// Creates a new `MultiFutureJoin` instance with the provided futures.
    ///
    /// # Arguments
    /// - `futures`: An array of futures to manage.
    ///
    /// # Returns
    /// A new `MultiFutureJoin` instance.
    pub fn new(futures: [Fut; N]) -> Self {
        Self {
            futures:   MaybeUninit::new(futures),
            all_error: SmallVec::new(),
            ready_map: [false; N],
            ok:        None,
        }
    }
}

impl<Fut, Ret, Error, const N: usize> MultiFutureRace<Fut, Ret, Error, N>
where
    Fut: Future<Output = Result<Ret, Error>>,
{
    /// Creates a new `MultiFutureRace` instance with the provided futures.
    ///
    /// # Arguments
    /// - `futures`: An array of futures to be polled.
    ///
    /// # Returns
    /// A new `MultiFutureRace` instance.
    pub fn new(futures: [Fut; N]) -> Self {
        Self {
            futures:   MaybeUninit::new(futures),
            all_error: SmallVec::new(),
            ready_map: [false; N],
            ok:        None,
        }
    }
}

impl<Fut, Ret, Error, const N: usize> MultiFutureRaceAnyError<Fut, Ret, Error, N>
where
    Fut: Future<Output = Result<Ret, Error>>,
{
    /// Creates a new `MultiFutureRaceAnyError` instance with the provided
    /// futures.
    ///
    /// # Arguments
    /// - `futures`: An array of futures to be polled.
    ///
    /// # Returns
    /// A new `MultiFutureRaceAnyError` instance.
    ///
    /// # Panics
    /// Panics if `N` is zero.
    pub const fn new(futures: [Fut; N]) -> Self {
        assert!(
            N > 0,
            "Cannot create a MultiFutureRaceAnyError with zero futures"
        );
        Self {
            futures:    MaybeUninit::new(futures),
            last_error: None,
            ready_map:  [false; N],
            ok:         None,
        }
    }
}

/// An error type for `MultiService` and `MultiFuture`.
///
/// - `NotReady`: Represents an error from a single service or future that is
///   not ready.
/// - `AllFailed`: Represents a collection of errors from all services or
///   futures that failed.
#[derive(Clone, Debug)]
pub enum MultiError<Error, const N: usize> {
    NotReady(Error),
    AllFailed(SmallVec<[Error; N]>),
}

impl<Error, const N: usize> Display for MultiError<Error, N>
where
    Error: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotReady(err) => write!(f, "NotReady({err})"),
            Self::AllFailed(errors) => {
                let mut first = true;
                write!(f, "AllFailed([")?;
                for error in errors {
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{error}")?;
                    first = false;
                }
                write!(f, "])")
            }
        }
    }
}

impl<Error, const N: usize> From<Error> for MultiError<Error, N> {
    fn from(error: Error) -> Self {
        Self::NotReady(error)
    }
}

impl<Error, const N: usize> From<SmallVec<[Error; N]>> for MultiError<Error, N> {
    fn from(error: SmallVec<[Error; N]>) -> Self {
        Self::AllFailed(error)
    }
}

unsafe fn iter_from_pin_maybeuninit<T, const N: usize>(
    slice: Pin<&mut MaybeUninit<[T; N]>>,
    ready_map: [bool; N],
) -> impl Iterator<Item = (usize, Pin<&mut T>)> {
    // Safety: `std` _could_ make this unsound if it were to decide Pin's
    // invariants aren't required to transmit through slices. Otherwise this has
    // the same safety as a normal field pin projection.
    unsafe { slice.get_unchecked_mut().assume_init_mut() }
        .iter_mut()
        .enumerate()
        .zip(ready_map)
        .filter_map(|((i, t), a)| {
            if a {
                None
            } else {
                Some((i, unsafe { Pin::new_unchecked(t) }))
            }
        })
}

unsafe fn take_array_from_pin_maybeuninit<T, const N: usize>(
    slice: Pin<&mut MaybeUninit<[T; N]>>,
    ready_map: [bool; N],
) -> impl Iterator<Item = T> {
    // Safety: `std` _could_ make this unsound if it were to decide Pin's
    // invariants aren't required to transmit through slices. Otherwise this has
    // the same safety as a normal field pin projection.
    //
    // Additionally, the caller must guarantee that the `ready_map` accurately
    // reflects the initialization state of the `MaybeUninit` array.
    let array = std::mem::replace(unsafe { slice.get_unchecked_mut() }, MaybeUninit::uninit());
    unsafe { array.assume_init() }
        .into_iter()
        .zip(ready_map)
        .filter_map(|(t, a)| if a { None } else { Some(t) })
}

impl<Fut, Ret, Error, const N: usize> Future for MultiFutureJoin<Fut, Ret, Error, N>
where
    Error: Send + 'static,
    Ret: Send + 'static,
    Fut: Future<Output = Result<Ret, Error>> + Send + 'static,
{
    type Output = Result<(Ret, JoinSet<Result<Ret, Error>>), MultiError<Error, N>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(response) = this.ok.take() {
            let mut set = JoinSet::new();
            for fut in unsafe { take_array_from_pin_maybeuninit(this.futures, *this.ready_map) } {
                set.spawn(fut);
            }
            while let Some(err) = this.all_error.pop() {
                set.spawn(futures_util::future::ready(Err(err)));
            }
            return Poll::Ready(Ok((response, set)));
        }
        for (index, fut) in unsafe { iter_from_pin_maybeuninit(this.futures, *this.ready_map) } {
            match fut.poll(cx) {
                Poll::Ready(Ok(response)) => {
                    *this.ok = Some(response);
                    this.ready_map[index] = true;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(Err(e)) => {
                    this.all_error.push(e);
                    this.ready_map[index] = true;
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }

        if this.ready_map.iter().all(|ready| *ready) {
            Poll::Ready(Err(std::mem::take(this.all_error).into()))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl<Fut, Ret, Error, const N: usize> Future for MultiFutureRace<Fut, Ret, Error, N>
where
    Error: Send + 'static,
    Ret: Send + 'static,
    Fut: Future<Output = Result<Ret, Error>> + Send + 'static,
{
    type Output = Result<Ret, MultiError<Error, N>>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(response) = this.ok.take() {
            for fut in unsafe { take_array_from_pin_maybeuninit(this.futures, *this.ready_map) } {
                tokio::spawn(fut);
            }
            return Poll::Ready(Ok(response));
        }
        for (index, fut) in unsafe { iter_from_pin_maybeuninit(this.futures, *this.ready_map) } {
            match fut.poll(cx) {
                Poll::Ready(Ok(response)) => {
                    *this.ok = Some(response);
                    this.ready_map[index] = true;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(Err(e)) => {
                    this.all_error.push(e);
                    this.ready_map[index] = true;
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }

        if this.ready_map.iter().all(|ready| *ready) {
            Poll::Ready(Err(std::mem::take(this.all_error).into()))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl<Fut, Ret, Error, const N: usize> Future for MultiFutureRaceAnyError<Fut, Ret, Error, N>
where
    Error: Send + 'static,
    Ret: Send + 'static,
    Fut: Future<Output = Result<Ret, Error>> + Send + 'static,
{
    type Output = Result<Ret, Error>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        if let Some(response) = this.ok.take() {
            for fut in unsafe { take_array_from_pin_maybeuninit(this.futures, *this.ready_map) } {
                tokio::spawn(fut);
            }
            return Poll::Ready(Ok(response));
        }
        for (index, fut) in unsafe { iter_from_pin_maybeuninit(this.futures, *this.ready_map) } {
            match fut.poll(cx) {
                Poll::Ready(Ok(response)) => {
                    *this.ok = Some(response);
                    this.ready_map[index] = true;
                    cx.waker().wake_by_ref();
                    return Poll::Pending;
                }
                Poll::Ready(Err(e)) => {
                    *this.last_error = Some(e);
                    this.ready_map[index] = true;
                }
                Poll::Pending => {
                    return Poll::Pending;
                }
            }
        }

        if this.ready_map.iter().all(|ready| *ready) {
            Poll::Ready(Err(this.last_error.take().expect("unreachable")))
        } else {
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

impl<S, const N: usize, Request, Response, Error, Fut> Service<Request> for MultiService<S, N>
where
    Response: Send + 'static,
    Request: Clone,
    Error: Send + 'static,
    Fut: Future<Output = Result<Response, Error>> + Send + 'static,
    S: Service<Request, Response = Response, Error = Error, Future = Fut>,
{
    type Error = MultiError<Error, N>;
    type Future = MultiFutureJoin<Fut, Response, Error, N>;
    type Response = (Response, JoinSet<Result<Response, Error>>);

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.inner {
            match service.poll_ready(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let futures = std::array::from_fn(|i| self.inner[i].call(req.clone()));
        MultiFutureJoin::new(futures)
    }
}

impl<S, const N: usize, Request, Response, Error, Fut> Service<Request> for MultiServiceRace<S, N>
where
    Response: Send + 'static,
    Request: Clone,
    Error: Send + 'static,
    Fut: Future<Output = Result<Response, Error>> + Send + 'static,
    S: Service<Request, Response = Response, Error = Error, Future = Fut>,
{
    type Error = MultiError<Error, N>;
    type Future = MultiFutureRace<Fut, Response, Error, N>;
    type Response = Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.inner {
            match service.poll_ready(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e.into())),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let futures = std::array::from_fn(|i| self.inner[i].call(req.clone()));
        MultiFutureRace::new(futures)
    }
}

impl<S, const N: usize, Request, Response, Error, Fut> Service<Request>
    for MultiServiceRaceAnyError<S, N>
where
    Response: Send + 'static,
    Request: Clone,
    Error: Send + 'static,
    Fut: Future<Output = Result<Response, Error>> + Send + 'static,
    S: Service<Request, Response = Response, Error = Error, Future = Fut>,
{
    type Error = Error;
    type Future = MultiFutureRaceAnyError<Fut, Response, Error, N>;
    type Response = Response;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        for service in &mut self.inner {
            match service.poll_ready(cx) {
                Poll::Ready(Ok(())) => {}
                Poll::Ready(Err(e)) => return Poll::Ready(Err(e)),
                Poll::Pending => return Poll::Pending,
            }
        }
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let futures = std::array::from_fn(|i| self.inner[i].call(req.clone()));
        MultiFutureRaceAnyError::new(futures)
    }
}

#[cfg(test)]
mod tests {

    use std::sync::atomic::{AtomicUsize, Ordering};

    use futures_util::future::BoxFuture;
    use triomphe::Arc;

    use crate::multi::{MultiError, MultiFutureJoin, MultiFutureRace};

    #[tokio::test]
    async fn multi_future_all_completed() {
        const N: usize = 100;
        type Future<'a> = BoxFuture<'a, Result<usize, ()>>;

        let completed_count = Arc::new(AtomicUsize::new(0));

        let futures: [Future; N] = std::array::from_fn(|n| {
            let completed_count = completed_count.clone();
            Box::pin(async move {
                completed_count.fetch_add(1, Ordering::SeqCst);
                Ok(n)
            }) as Future
        });

        let multi = MultiFutureJoin::new(futures);
        let (result, set) = multi.await.unwrap();
        let map: [usize; N] = std::array::from_fn(|i| i);
        assert!(map.contains(&result));
        set.join_all().await;
        assert_eq!(completed_count.load(Ordering::SeqCst), N);
    }

    #[tokio::test]
    async fn multi_future_all_failed() {
        const N: usize = 50;
        type Future<'a> = BoxFuture<'a, Result<usize, ()>>;

        let completed_count = Arc::new(AtomicUsize::new(0));

        let futures: [Future; N] = std::array::from_fn(|_| {
            let completed_count = completed_count.clone();
            Box::pin(async move {
                completed_count.fetch_add(1, Ordering::SeqCst);
                Err(())
            }) as Future
        });

        let multi = MultiFutureJoin::new(futures);
        assert!(matches!(multi.await, Err(MultiError::AllFailed(_))));
        assert_eq!(completed_count.load(Ordering::SeqCst), N);
    }

    #[tokio::test]
    async fn multi_future_any_failed() {
        const N: usize = 4;
        type Future<'a> = BoxFuture<'a, Result<usize, ()>>;

        let completed_count = Arc::new(AtomicUsize::new(0));

        let futures: [Future; N] = std::array::from_fn(|i| {
            let completed_count = completed_count.clone();
            Box::pin(async move {
                completed_count.fetch_add(1, Ordering::SeqCst);
                if i % 2 == 0 { Ok(i) } else { Err(()) }
            }) as Future
        });

        let multi = MultiFutureJoin::new(futures);
        let Ok((result, set)) = multi.await else {
            panic!("Unexpected error");
        };

        assert!([0, 2].contains(&result));
        set.join_all().await;
        assert_eq!(completed_count.load(Ordering::SeqCst), N);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn multi_future_race_any_failed() {
        const N: usize = 4;
        type Future<'a> = BoxFuture<'a, Result<usize, ()>>;

        let completed_count = Arc::new(AtomicUsize::new(0));

        let futures: [Future; N] = std::array::from_fn(|i| {
            let completed_count = completed_count.clone();
            Box::pin(async move {
                completed_count.fetch_add(1, Ordering::SeqCst);
                if i % 2 == 0 { Ok(i) } else { Err(()) }
            }) as Future
        });

        let multi = MultiFutureRace::new(futures);
        let Ok(result) = multi.await else {
            panic!("Unexpected error");
        };

        assert!([0, 2].contains(&result));
        loop {
            if completed_count.load(Ordering::SeqCst) == N {
                break;
            }
            tokio::task::yield_now().await;
        }
    }
}
