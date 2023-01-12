#![cfg(feature = "tokio_io")]

use std::error;

use futures_util_either::Either;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt as _},
    net::{TcpStream, UnixStream},
};

async fn rw() -> Result<(), Box<dyn error::Error>> {
    let mut stream = if true {
        Either::Right(UnixStream::connect("TODO").await?)
    } else {
        Either::Left(TcpStream::connect("127.0.0.1:8080").await?)
    };

    stream.write_all(b"foo").await?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;

    Ok(())
}

#[cfg(feature = "std")]
#[tokio::test]
#[should_panic(expected = "No such file or directory")]
async fn impl_tokio_io() {
    rw().await.unwrap();
}
