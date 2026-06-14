use crate::runtime::{Runtime, TryRecvError};

pub type OneshotSenderOf<R, T> = <<R as Runtime>::Oneshot as Oneshot>::Sender<T>;
pub type OneshotReceiverOf<R, T> = <<R as Runtime>::Oneshot as Oneshot>::Receiver<T>;

pub trait Oneshot {
    type Receiver<T: Send>: Receiver<T>;
    type Sender<T: Send>: Sender<T>;

    fn channel<T: Send>() -> (Self::Sender<T>, Self::Receiver<T>);
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
