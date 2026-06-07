use std::{
    net::{Ipv4Addr, SocketAddrV4},
    time::Duration,
};

use remyx::runtime::{
    Runtime,
    tcp::{Listener, Read, Stream, Tcp, Write},
    time::Time,
};

pub async fn listener_bind_succeeds_on_localhost<R: Runtime>() {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8080);
    let listener = R::Tcp::listener(addr).await;
    assert!(listener.is_ok());
}

pub async fn stream_connect_succeeds_to_bound_listener<R: Runtime>() {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8081);
    R::spawn_local(async move {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8081);
        let listener = R::Tcp::listener(addr).await.unwrap();

        while listener.accept().await.is_ok() {}
    });
    R::Time::sleep(Duration::from_millis(250)).await;
    let stream = R::Tcp::stream(addr).await;
    assert!(stream.is_ok());
}

pub async fn stream_read_write_round_trip_succeeds<R: Runtime>() {
    let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8082);
    R::spawn_local(async move {
        let addr = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 8082);
        let listener = R::Tcp::listener(addr).await.unwrap();
        while let Ok((stream, _)) = listener.accept().await {
            let (mut read, mut write) = stream.split();
            write.write_all("hello world!".as_bytes()).await.unwrap();

            let mut buf = [0; 12];
            read.read(&mut buf).await.unwrap();
            assert_eq!("hello there!", String::from_utf8_lossy(&buf));
        }
    });

    R::Time::sleep(Duration::from_millis(250)).await;

    let (mut read, mut write) = R::Tcp::stream(addr).await.unwrap().split();

    let mut buf = [0; 12];
    read.read(&mut buf).await.unwrap();
    assert_eq!("hello world!", String::from_utf8_lossy(&buf));

    write.write_all("hello there!".as_bytes()).await.unwrap();
}
