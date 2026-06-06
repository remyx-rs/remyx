use std::io;

use crate::stream::LocalBoxFusedStream;
use futures::StreamExt;

pub fn events() -> LocalBoxFusedStream<io::Result<crossterm::event::Event>> {
    Box::pin(crossterm::event::EventStream::new().fuse())
}
