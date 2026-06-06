use std::{
    pin::{Pin, pin},
    thread::{self},
};

use futures::FutureExt;

use crate::{
    runtime::mpsc::{Mpsc, MpscSenderOf, Receiver as MpscReceiver, Sender as MpscSender},
    runtime::oneshot::{
        Oneshot, OneshotReceiverOf, Receiver as OneshotReceiver, Sender as OneshotSender,
        TryRecvError,
    },
    runtime::tcp::Tcp,
    runtime::time::Time,
};

#[cfg(feature = "tokio")]
pub mod tokio;

pub mod mpsc;
pub mod oneshot;
pub mod tcp;
pub mod time;

type Callback = Pin<Box<dyn Future<Output = ()> + Send>>;

pub type JoinHandleOf<R, T> = <R as Runtime>::JoinHandle<T>;

pub enum JoinError {
    Panicked,
    Cancelled,
}

pub trait JoinHandle<T>: Send {
    fn cancel(&self);
    fn is_finished(&self) -> bool;
    fn into_future(self) -> impl Future<Output = Result<T, JoinError>>;
}

pub trait Runtime: 'static {
    type JoinHandle<T: Send>: JoinHandle<T>;
    type Mpsc: Mpsc;
    type Oneshot: Oneshot;
    type Tcp: Tcp;
    type Time: Time;

    fn new(threads: usize) -> Self;

    fn spawn<F>(fut: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static;

    fn spawn_local<F>(fut: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send + 'static;

    fn block_on<Fut>(&self, fut: Fut) -> Fut::Output
    where
        Fut: Future;

    fn defer<Fut>(threads: usize, capacity: usize, fut: Fut) -> Handle<Self, Fut>
    where
        Self: Sized,
        Fut: Future + Send + 'static,
        Fut::Output: Send,
    {
        let (tx, mut rx) = Self::Mpsc::channel::<Callback>(capacity);
        let (tx_fut, rx_fut) = Self::Oneshot::channel::<Option<Fut::Output>>();
        thread::spawn(move || {
            let runtime = Self::new(threads);
            runtime.block_on(async move {
                let mut fut = pin!(fut.fuse());
                let output = loop {
                    futures::select! {
                        callback = rx.recv().fuse() => {
                            match callback {
                                Ok(callback) => {
                                    callback.await;
                                }
                                // Handle dropped
                                Err(_) => {
                                    break None;
                                }
                            }
                        }
                       result = fut => {
                           // Block on future returned
                           break Some(result);
                        }
                    }
                };

                if let Err(_err) = tx_fut.send(output) {}
            });
        });

        Handle::new(tx, rx_fut)
    }
}

pub struct Handle<R, Fut>
where
    Fut: Future,
    Fut::Output: Send,
    R: Runtime,
{
    tx: MpscSenderOf<R, Callback>,
    rx: OneshotReceiverOf<R, Option<Fut::Output>>,
}

impl<R, Fut> Handle<R, Fut>
where
    Fut: Future,
    Fut::Output: Send,
    R: Runtime,
{
    fn new(tx: MpscSenderOf<R, Callback>, rx: OneshotReceiverOf<R, Option<Fut::Output>>) -> Self {
        Self { tx, rx }
    }

    pub async fn spawn<F>(&self, f: F) -> Result<JoinHandleOf<R, F::Output>, oneshot::TryRecvError>
    where
        F: Future + Send + 'static,
        F::Output: Send,
    {
        let (tx, mut rx) = R::Oneshot::channel::<JoinHandleOf<R, F::Output>>();
        let callback: Callback = Box::pin(async move {
            let handle = R::spawn(f);
            let _ = tx.send(handle);
        });

        self.tx.send(callback).await.map_err(|err| match err {
            mpsc::TrySendError::Full(_) => unreachable!(),
            mpsc::TrySendError::Closed(_) => oneshot::TryRecvError::Disconnected,
        })?;

        rx.recv().await
    }

    pub fn join(&mut self) -> impl Future<Output = Result<Fut::Output, TryRecvError>> {
        self.rx.recv().map(|res| match res {
            Ok(Some(val)) => Ok(val),
            Ok(None) => Err(TryRecvError::Empty),
            Err(err) => Err(err),
        })
    }
}
