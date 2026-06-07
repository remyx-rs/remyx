use futures::FutureExt;
use tokio::sync::mpsc::error::SendError;

use crate::runtime::mpsc::{Mpsc, Receiver, Sender, TryRecvError, TrySendError};

pub struct TokioMpsc {}

impl Mpsc for TokioMpsc {
    type Receiver<T: Send> = tokio::sync::mpsc::Receiver<T>;

    type Sender<T: Send> = tokio::sync::mpsc::Sender<T>;

    fn channel<T: Send>(size: usize) -> (Self::Sender<T>, Self::Receiver<T>) {
        tokio::sync::mpsc::channel(size)
    }
}

impl<T: Send> Sender<T> for tokio::sync::mpsc::Sender<T> {
    fn send(&self, value: T) -> impl Future<Output = Result<(), TrySendError<T>>> {
        self.send(value).map(|res| match res {
            Ok(_) => Ok(()),
            Err(SendError(val)) => Err(TrySendError::Closed(val)),
        })
    }
}

impl<T: Send> Receiver<T> for tokio::sync::mpsc::Receiver<T> {
    fn recv(&mut self) -> impl Future<Output = Result<T, TryRecvError>> {
        self.recv().map(|res| match res {
            Some(value) => Ok(value),
            None => Err(TryRecvError::Disconnected),
        })
    }
}
