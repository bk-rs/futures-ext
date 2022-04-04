use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io::{Result, SeekFrom};

use tokio::io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite, ReadBuf};

use super::Either;

//
impl<A, B> AsyncRead for Either<A, B>
where
    A: AsyncRead,
    B: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<Result<()>> {
        match self.project() {
            Either::Left(x) => x.poll_read(cx, buf),
            Either::Right(x) => x.poll_read(cx, buf),
        }
    }
}

//
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

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.project() {
            Either::Left(x) => x.poll_flush(cx),
            Either::Right(x) => x.poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<()>> {
        match self.project() {
            Either::Left(x) => x.poll_shutdown(cx),
            Either::Right(x) => x.poll_shutdown(cx),
        }
    }
}

//
impl<A, B> AsyncSeek for Either<A, B>
where
    A: AsyncSeek,
    B: AsyncSeek,
{
    fn start_seek(self: Pin<&mut Self>, position: SeekFrom) -> Result<()> {
        match self.project() {
            Either::Left(x) => x.start_seek(position),
            Either::Right(x) => x.start_seek(position),
        }
    }

    fn poll_complete(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<u64>> {
        match self.project() {
            Either::Left(x) => x.poll_complete(cx),
            Either::Right(x) => x.poll_complete(cx),
        }
    }
}

//
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
