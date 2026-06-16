use std::{fmt, marker::PhantomData, pin::Pin};

use crate::runtime::{mpsc::Mpsc, oneshot::Oneshot, tcp::Tcp, time::Time};

#[cfg(feature = "tokio")]
pub mod tokio;

pub mod mpsc;
pub mod oneshot;
pub mod tcp;
pub mod time;

pub type JoinHandleOf<R, T> = <R as Runtime>::JoinHandle<T>;
pub type JoinHandleFutOf<R, T> = JoinHandleFut<JoinHandleOf<R, T>, T>;

pub enum JoinError {
    Panicked,
    Cancelled,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    Empty,
    Disconnected,
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TryRecvError::Empty => "receiving on an empty channel".fmt(fmt),
            TryRecvError::Disconnected => "receiving on a closed channel".fmt(fmt),
        }
    }
}

pub trait JoinHandle<T>: Unpin + Send {
    fn cancel(&self);
    fn is_finished(&self) -> bool;

    fn into_future(self) -> JoinHandleFut<Self, T>
    where
        Self: Sized,
    {
        JoinHandleFut::new(self)
    }

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<T, JoinError>>;
}

pub struct JoinHandleFut<JoinHandle, T> {
    handle: JoinHandle,
    _marker: PhantomData<fn() -> T>,
}

impl<JoinHandle, T> JoinHandleFut<JoinHandle, T>
where
    JoinHandle: self::JoinHandle<T>,
{
    pub fn new(handle: JoinHandle) -> Self {
        Self {
            handle,
            _marker: PhantomData,
        }
    }
}

impl<JoinHandle, T> Future for JoinHandleFut<JoinHandle, T>
where
    JoinHandle: self::JoinHandle<T>,
{
    type Output = Result<T, JoinError>;

    fn poll(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        let this = self.get_mut();
        Pin::new(&mut this.handle).poll(cx)
    }
}

pub trait Runtime: 'static {
    type JoinHandle<T: Send>: JoinHandle<T>;
    type Mpsc: Mpsc;
    type Oneshot: Oneshot;
    type Tcp: Tcp;
    type Time: Time;

    fn new(threads: usize) -> Self;

    fn spawn<F>(&self, fut: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn block_on<Fut>(&self, fut: Fut) -> Fut::Output
    where
        Fut: Future;
}
