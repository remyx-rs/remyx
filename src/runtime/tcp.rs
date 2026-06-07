use std::{io, net::SocketAddr};

use crate::runtime::Runtime;

pub type TcpStreamOf<R> = <<R as Runtime>::Tcp as Tcp>::Stream;
pub type TcpStreamReadOf<R> = <<<R as Runtime>::Tcp as Tcp>::Stream as Stream>::Read;
pub type TcpStreamWriteOf<R> = <<<R as Runtime>::Tcp as Tcp>::Stream as Stream>::Write;
pub type TcpListenerOf<R> = <<R as Runtime>::Tcp as Tcp>::Listener;

pub trait Tcp {
    type Listener: Listener<Self::Stream>;
    type Stream: Stream;

    fn stream(addr: impl Into<SocketAddr>) -> impl Future<Output = io::Result<Self::Stream>>;
    fn listener(addr: impl Into<SocketAddr>) -> impl Future<Output = io::Result<Self::Listener>>;
}

pub trait Listener<S: Stream> {
    fn accept(&self) -> impl Future<Output = io::Result<(S, SocketAddr)>>;
}

pub trait Stream: Read + Write + State {
    type Read: Read;
    type Write: Write;

    fn split(self) -> (Self::Read, Self::Write);
}

pub trait Read: State {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> impl Future<Output = io::Result<usize>> + 'a;
}

pub trait Write: State {
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = io::Result<()>> + 'a;
}

pub trait State {
    fn local_addr(&self) -> io::Result<SocketAddr>;
    fn peer_addr(&self) -> io::Result<SocketAddr>;
}
