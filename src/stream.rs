use std::pin::Pin;

pub type LocalBoxFusedStream<O> = Pin<Box<dyn futures::stream::FusedStream<Item = O>>>;
