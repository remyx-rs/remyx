use futures::FutureExt;

use crate::runtime::{
    TryRecvError,
    oneshot::{Oneshot, Receiver, Sender},
};

pub struct TokioOneshot {}

impl Oneshot for TokioOneshot {
    type Receiver<T: Send> = tokio::sync::oneshot::Receiver<T>;
    type Sender<T: Send> = tokio::sync::oneshot::Sender<T>;

    fn channel<T: Send>() -> (Self::Sender<T>, Self::Receiver<T>) {
        tokio::sync::oneshot::channel()
    }
}

impl<T: Send> Sender<T> for tokio::sync::oneshot::Sender<T> {
    fn send(self, value: T) -> Result<(), T> {
        self.send(value)
    }
}

impl<T: Send> Receiver<T> for tokio::sync::oneshot::Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, TryRecvError>> {
        self.map(|res| match res {
            Ok(val) => Ok(val),
            Err(_) => Err(TryRecvError::Disconnected),
        })
    }
}
