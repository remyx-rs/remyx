use std::{io, net::SocketAddr};

use crate::runtime::tcp::{Listener, Read, State, Stream, Tcp, Write};

pub struct TokioTcp {}

impl Tcp for TokioTcp {
    type Listener = tokio::net::TcpListener;
    type Stream = tokio::net::TcpStream;

    fn stream(
        addr: impl Into<std::net::SocketAddr>,
    ) -> impl Future<Output = std::io::Result<Self::Stream>> {
        tokio::net::TcpStream::connect(addr.into())
    }

    fn listener(
        addr: impl Into<std::net::SocketAddr>,
    ) -> impl Future<Output = std::io::Result<Self::Listener>> {
        tokio::net::TcpListener::bind(addr.into())
    }
}

impl Stream for tokio::net::TcpStream {
    type Read = tokio::net::tcp::OwnedReadHalf;
    type Write = tokio::net::tcp::OwnedWriteHalf;

    fn split(self) -> (Self::Read, Self::Write) {
        self.into_split()
    }
}

impl Read for tokio::net::TcpStream {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> impl Future<Output = io::Result<usize>> + 'a {
        tokio::io::AsyncReadExt::read(self, buf)
    }
}

impl Write for tokio::net::TcpStream {
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = io::Result<()>> + 'a {
        tokio::io::AsyncWriteExt::write_all(self, buf)
    }
}

impl State for tokio::net::TcpStream {
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }
}

impl Read for tokio::net::tcp::OwnedReadHalf {
    fn read<'a>(&'a mut self, buf: &'a mut [u8]) -> impl Future<Output = io::Result<usize>> + 'a {
        tokio::io::AsyncReadExt::read(self, buf)
    }
}

impl State for tokio::net::tcp::OwnedReadHalf {
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }
}

impl Write for tokio::net::tcp::OwnedWriteHalf {
    fn write_all<'a>(&'a mut self, buf: &'a [u8]) -> impl Future<Output = io::Result<()>> + 'a {
        tokio::io::AsyncWriteExt::write_all(self, buf)
    }
}

impl State for tokio::net::tcp::OwnedWriteHalf {
    fn local_addr(&self) -> io::Result<SocketAddr> {
        self.local_addr()
    }

    fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.peer_addr()
    }
}

impl Listener<tokio::net::TcpStream> for tokio::net::TcpListener {
    fn accept(&self) -> impl Future<Output = io::Result<(tokio::net::TcpStream, SocketAddr)>> {
        self.accept()
    }
}
