use std::fmt;

use crate::runtime::Runtime;

pub type OneshotSenderOf<R, T> = <<R as Runtime>::Oneshot as Oneshot>::Sender<T>;
pub type OneshotReceiverOf<R, T> = <<R as Runtime>::Oneshot as Oneshot>::Receiver<T>;

pub trait Oneshot {
    type Receiver<T: Send>: Receiver<T>;
    type Sender<T: Send>: Sender<T>;

    fn channel<T: Send>() -> (Self::Sender<T>, Self::Receiver<T>);
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

pub trait Sender<T>: Send
where
    T: Send,
{
    fn send(self, value: T) -> Result<(), T>;
}

pub trait Receiver<T: Send>: Send {
    fn recv(&mut self) -> impl Future<Output = Result<T, TryRecvError>>;
}
