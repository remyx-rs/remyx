use std::fmt;

use crate::runtime::Runtime;

pub type MpscSenderOf<R, T> = <<R as Runtime>::Mpsc as Mpsc>::Sender<T>;
pub type MpscReceiverOf<R, T> = <<R as Runtime>::Mpsc as Mpsc>::Receiver<T>;

pub trait Mpsc {
    type Receiver<T: Send>: Receiver<T>;
    type Sender<T: Send>: Sender<T>;

    fn channel<T: Send>(size: usize) -> (Self::Sender<T>, Self::Receiver<T>);
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TrySendError<T> {
    Full(T),
    Closed(T),
}

impl<T> TrySendError<T> {
    pub fn into_inner(self) -> T {
        match self {
            TrySendError::Full(val) => val,
            TrySendError::Closed(val) => val,
        }
    }
}

impl<T> fmt::Debug for TrySendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TrySendError::Full(..) => "Full(..)".fmt(f),
            TrySendError::Closed(..) => "Closed(..)".fmt(f),
        }
    }
}

impl<T> fmt::Display for TrySendError<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            fmt,
            "{}",
            match self {
                TrySendError::Full(..) => "no available capacity",
                TrySendError::Closed(..) => "channel closed",
            }
        )
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TryRecvError {
    /// This **channel** is currently empty, but the **Sender**(s) have not yet
    /// disconnected, so data may yet become available.
    Empty,
    /// The **channel**'s sending half has become disconnected, and there will
    /// never be any more data received on it.
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

pub trait Sender<T>: Send
where
    T: Send,
{
    fn send(&self, value: T) -> impl Future<Output = Result<(), TrySendError<T>>> + Send;
}

pub trait Receiver<T: Send>: Send {
    fn recv(&mut self) -> impl Future<Output = Result<T, TryRecvError>>;
}
