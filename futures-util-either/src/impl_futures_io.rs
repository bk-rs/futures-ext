use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io::Error as IoError;

use futures_io::{AsyncRead, IoSliceMut};

use super::Either;

//
// Ref https://github.com/rust-lang/futures-rs/blob/0.3.21/futures-util/src/future/either.rs#L191
impl<A, B> AsyncRead for Either<A, B>
where
    A: AsyncRead,
    B: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize, IoError>> {
        match self.project() {
            Either::Left(x) => x.poll_read(cx, buf),
            Either::Right(x) => x.poll_read(cx, buf),
        }
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<Result<usize, IoError>> {
        match self.project() {
            Either::Left(x) => x.poll_read_vectored(cx, bufs),
            Either::Right(x) => x.poll_read_vectored(cx, bufs),
        }
    }
}
