use std::{
    pin::Pin,
    task::{Context, Poll},
};

use futures::{StreamExt, stream::BoxStream};

use crate::runtime::{
    self,
    mpsc::{Mpsc, MpscSenderOf, Receiver, Sender, TrySendError},
};

/// A stream splitter that broadcasts items to multiple subscribers.
///
/// Items are delivered on a best-effort basis: if a subscriber's channel is full,
/// the item is dropped for that subscriber rather than blocking the producer.
/// Subscribers whose channels are closed are automatically removed.
pub struct Tee<Runtime, Item>
where
    Item: Send,
    Runtime: runtime::Runtime,
{
    subscribers: Vec<MpscSenderOf<Runtime, Item>>,
    stream: BoxStream<'static, Item>,
}

impl<Runtime, Item> Tee<Runtime, Item>
where
    Item: Send,
    Runtime: runtime::Runtime,
{
    pub fn new(stream: BoxStream<'static, Item>) -> Self {
        Self {
            subscribers: Default::default(),
            stream,
        }
    }

    pub fn fork(&mut self) -> impl futures::Stream<Item = Item> + use<Runtime, Item> {
        let (tx, rx) = Runtime::Mpsc::channel(1024);
        self.subscribers.push(tx);

        futures::stream::unfold(rx, |mut rx| async move {
            match rx.recv().await {
                Ok(item) => Some((item, rx)),
                Err(_) => None,
            }
        })
    }
}

impl<Runtime, Item> futures::Stream for Tee<Runtime, Item>
where
    Item: Send + Clone,
    Runtime: runtime::Runtime,
{
    type Item = Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        let Some(item) = futures::ready!(this.stream.poll_next_unpin(cx)) else {
            return Poll::Ready(None);
        };

        let mut index = 0;
        while index < this.subscribers.len() {
            let subscriber = &this.subscribers[index];

            match subscriber.try_send(item.clone()) {
                Err(TrySendError::Closed(_)) => {
                    this.subscribers.remove(index);
                }
                Ok(_) | Err(TrySendError::Full(_)) => {
                    index += 1;
                }
            }
        }

        Poll::Ready(Some(item))
    }
}

impl<Runtime, Item> futures::stream::FusedStream for Tee<Runtime, Item>
where
    Item: Send + Clone,
    Runtime: runtime::Runtime,
{
    fn is_terminated(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod test {
    use std::{pin::pin, time::Duration};

    use futures::StreamExt;

    use crate::{
        runtime::{
            Runtime,
            time::Time,
            tokio::{Tokio, time::TokioTime},
        },
        stream::Tee,
    };

    #[test]
    pub fn tee_stream_test_original_and_fork_receive_produced_messages() {
        Tokio::new(0).block_on(async move {
            let stream = futures::stream::unfold(100, |val| async move {
                TokioTime::sleep(Duration::from_millis(val)).await;
                Some((val, val * 2))
            })
            .boxed();

            let mut tee = Tee::<Tokio, _>::new(stream);

            let mut subscriber = pin!(tee.fork());
            assert_eq!(tee.next().await, Some(100));
            assert_eq!(tee.next().await, Some(200));
            assert_eq!(subscriber.next().await, Some(100));
            assert_eq!(subscriber.next().await, Some(200));
        })
    }
}
