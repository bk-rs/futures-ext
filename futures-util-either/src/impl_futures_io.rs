use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_io::{
    AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite, IoSlice, IoSliceMut, Result, SeekFrom,
};

use super::Either;

//
// https://github.com/rust-lang/futures-rs/blob/0.3.21/futures-util/src/future/either.rs#L191-L296
//
impl<A, B> AsyncRead for Either<A, B>
where
    A: AsyncRead,
    B: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<Result<usize>> {
        match self.project() {
            Either::Left(x) => x.poll_read(cx, buf),
            Either::Right(x) => x.poll_read(cx, buf),
        }
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<Result<usize>> {
        match self.project() {
            Either::Left(x) => x.poll_read_vectored(cx, bufs),
            Either::Right(x) => x.poll_read_vectored(cx, bufs),
        }
    }
}

impl<A, B> AsyncWrite for Either<A, B>
where
    A: AsyncWrite,
    B: AsyncWrite,
{
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<Result<usize>> {
        match self.project() {
            Either::Left(x) => x.poll_write(cx, buf),
            Either::Right(x) => x.poll_write(cx, buf),
        }
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<Result<usize>> {
        match self.project() {
            Either::Left(x) => x.poll_write_vectored(cx, bufs),
            Either::Right(x) => x.poll_write_vectored(cx, bufs),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.project() {
            Either::Left(x) => x.poll_flush(cx),
            Either::Right(x) => x.poll_flush(cx),
        }
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.project() {
            Either::Left(x) => x.poll_close(cx),
            Either::Right(x) => x.poll_close(cx),
        }
    }
}

impl<A, B> AsyncSeek for Either<A, B>
where
    A: AsyncSeek,
    B: AsyncSeek,
{
    fn poll_seek(self: Pin<&mut Self>, cx: &mut Context<'_>, pos: SeekFrom) -> Poll<Result<u64>> {
        match self.project() {
            Either::Left(x) => x.poll_seek(cx, pos),
            Either::Right(x) => x.poll_seek(cx, pos),
        }
    }
}

impl<A, B> AsyncBufRead for Either<A, B>
where
    A: AsyncBufRead,
    B: AsyncBufRead,
{
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<&[u8]>> {
        match self.project() {
            Either::Left(x) => x.poll_fill_buf(cx),
            Either::Right(x) => x.poll_fill_buf(cx),
        }
    }

    fn consume(self: Pin<&mut Self>, amt: usize) {
        match self.project() {
            Either::Left(x) => x.consume(amt),
            Either::Right(x) => x.consume(amt),
        }
    }
}
