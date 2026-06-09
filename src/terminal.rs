use std::io;

use crate::{runtime, stream};
use crossterm::event::{Event, EventStream};
use futures::{StreamExt, TryStreamExt};

pub type EventResult = Result<Event, io::ErrorKind>;

pub fn events<Runtime: runtime::Runtime>() -> stream::Tee<Runtime, Result<Event, io::ErrorKind>> {
    let stream = EventStream::new().map_err(|err| err.kind()).boxed();
    stream::Tee::new(stream)
}
