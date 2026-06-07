use tokio::runtime::LocalOptions;

use crate::runtime::{
    JoinError, JoinHandle, Runtime,
    tokio::{mpsc::TokioMpsc, oneshot::TokioOneshot, tcp::TokioTcp, time::TokioTime},
};

pub mod mpsc;
pub mod oneshot;
pub mod tcp;
pub mod time;

pub enum Tokio {
    SingleThreaded { rt: tokio::runtime::LocalRuntime },
    MultiThreaded { rt: tokio::runtime::Runtime },
}

impl<T: Send> JoinHandle<T> for tokio::task::JoinHandle<T> {
    fn cancel(&self) {
        self.abort();
    }

    fn is_finished(&self) -> bool {
        self.is_finished()
    }

    fn poll(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<T, JoinError>> {
        std::future::Future::poll(self, cx).map_err(|err| {
            if err.is_panic() {
                JoinError::Panicked
            } else {
                JoinError::Cancelled
            }
        })
    }
}

impl Runtime for Tokio {
    type JoinHandle<T: Send> = tokio::task::JoinHandle<T>;
    type Mpsc = TokioMpsc;
    type Oneshot = TokioOneshot;
    type Tcp = TokioTcp;
    type Time = TokioTime;

    fn new(threads: usize) -> Self {
        if threads > 1 {
            Tokio::MultiThreaded {
                rt: tokio::runtime::Builder::new_multi_thread()
                    .enable_all()
                    .build()
                    .unwrap(),
            }
        } else {
            Tokio::SingleThreaded {
                rt: tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build_local(LocalOptions::default())
                    .unwrap(),
            }
        }
    }

    fn spawn<F>(fut: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        tokio::spawn(fut)
    }

    fn block_on<Fut>(&self, fut: Fut) -> Fut::Output
    where
        Fut: Future,
    {
        match self {
            Tokio::SingleThreaded { rt } => rt.block_on(fut),
            Tokio::MultiThreaded { rt } => rt.block_on(fut),
        }
    }

    fn spawn_local<F>(fut: F) -> Self::JoinHandle<F::Output>
    where
        F: Future + 'static,
        F::Output: Send + 'static,
    {
        tokio::task::spawn_local(fut)
    }
}
