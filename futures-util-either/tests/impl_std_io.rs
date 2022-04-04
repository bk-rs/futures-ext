#![cfg(feature = "std")]

use std::{
    error,
    io::{Read as _, Write as _},
    net::TcpStream,
    os::unix::net::UnixStream,
};

use futures_util_either::Either;

fn rw() -> Result<(), Box<dyn error::Error>> {
    let mut stream = if true {
        Either::Right(UnixStream::connect("TODO")?)
    } else {
        Either::Left(TcpStream::connect("127.0.0.1:34254")?)
    };

    stream.write_all(b"foo")?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf)?;

    Ok(())
}

#[test]
#[should_panic(expected = "No such file or directory")]
fn impl_std_io() {
    rw().unwrap();
}
