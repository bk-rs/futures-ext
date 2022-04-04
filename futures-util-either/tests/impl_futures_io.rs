#![cfg(feature = "futures_io")]

use std::{error, net::TcpStream, os::unix::net::UnixStream};

use async_io::Async;
use futures_util::io::{AsyncReadExt as _, AsyncWriteExt as _};
use futures_util_either::Either;

async fn rw() -> Result<(), Box<dyn error::Error>> {
    let mut stream = if true {
        Either::Right(Async::<UnixStream>::connect("TODO").await?)
    } else {
        Either::Left(Async::<TcpStream>::connect(([127, 0, 0, 1], 8000)).await?)
    };

    stream.write_all(b"foo").await?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await?;

    Ok(())
}

#[test]
#[should_panic(expected = "No such file or directory")]
fn impl_futures_io() {
    futures_executor::block_on(async {
        rw().await.unwrap();
    })
}
