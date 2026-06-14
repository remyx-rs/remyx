use std::{io, net::SocketAddr};

use crate::runtime::Runtime;

pub type TcpStreamOf<R> = <<R as Runtime>::Tcp as Tcp>::Stream;
pub type TcpStreamReadOf<R> = <<<R as Runtime>::Tcp as Tcp>::Stream as Stream>::Read;
pub type TcpStreamWriteOf<R> = <<<R as Runtime>::Tcp as Tcp>::Stream as Stream>::Write;
pub type TcpListenerOf<R> = <<R as Runtime>::Tcp as Tcp>::Listener;

pub trait Tcp {
    type Listener: Listener<Self::Stream>;
    type Stream: Stream;

    fn stream(addr: impl Into<SocketAddr>)
    -> impl Future<Output = io::Result<Self::Stream>> + Send;
    fn listener(
        addr: impl Into<SocketAddr>,
    ) -> impl Future<Output = io::Result<Self::Listener>> + Send;
}

pub trait Listener<S: Stream>: Send {
    fn accept(&self) -> impl Future<Output = io::Result<(S, SocketAddr)>> + Send;
    fn local_addr(&self) -> io::Result<SocketAddr>;
}

pub trait Stream: Read + Write + State {
    type Read: Read;
    type Write: Write;

    fn split(self) -> (Self::Read, Self::Write);
}

pub trait Read: State + Send {
    fn read<'a>(
        &'a mut self,
        buf: &'a mut [u8],
    ) -> impl Future<Output = io::Result<usize>> + Send + 'a;
}

pub trait Write: State + Send {
    fn write_all<'a>(
        &'a mut self,
        buf: &'a [u8],
    ) -> impl Future<Output = io::Result<()>> + Send + 'a;
}

pub trait State {
    fn local_addr(&self) -> io::Result<SocketAddr>;
    fn peer_addr(&self) -> io::Result<SocketAddr>;
}
