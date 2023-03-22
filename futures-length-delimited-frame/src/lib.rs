use core::{
    pin::Pin,
    task::{Context, Poll},
};
use std::io::{Error as IoError, ErrorKind as IoErrorKind};

use futures_core::{ready, Stream};
use futures_io::{AsyncRead, AsyncWrite};
use futures_sink::Sink;
use pin_project_lite::pin_project;

//
pin_project! {
    #[derive(Debug)]
    pub struct Decoder<R> {
        #[pin]
        reader: R,
        buf: Vec<u8>,
        n_read: usize,
    }
}

//
pin_project! {
    #[derive(Debug)]
    pub struct Encoder<W> {
        #[pin]
        inner: W,
        buf: Vec<u8>,
    }
}

impl<W: AsyncWrite> Encoder<W> {
    pub fn new(inner: W) -> Self {
        Self::with_capacity(2048, inner)
    }

    pub fn with_capacity(cap: usize, inner: W) -> Self {
        Self {
            inner,
            buf: Vec::with_capacity(cap),
        }
    }

    pub fn get_ref(&self) -> &W {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }

    pub fn into_inner(self) -> W {
        self.inner
    }
}

// https://github.com/tokio-rs/tokio/blob/tokio-util-0.7.7/tokio-util/src/codec/framed_impl.rs#L253
impl<T: AsRef<[u8]>, W: AsyncWrite> Sink<T> for Encoder<W> {
    type Error = IoError;

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        if !self.buf.is_empty() {
            <Encoder<W> as Sink<T>>::poll_flush(self.as_mut(), cx)
        } else {
            Poll::Ready(Ok(()))
        }
    }

    fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
        let this = self.project();

        let data = item.as_ref();
        let length = data.len() as u64;

        this.buf.extend_from_slice(length.to_be_bytes().as_ref());
        this.buf.extend_from_slice(data);

        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut this = self.project();

        let mut n_write = 0;
        while !this.buf[n_write..].is_empty() {
            let n = ready!(this.inner.as_mut().poll_write(cx, &this.buf[n_write..]))?;
            n_write += n;

            if n == 0 {
                return Poll::Ready(Err(IoErrorKind::WriteZero.into()));
            }
        }

        ready!(this.inner.as_mut().poll_flush(cx))?;

        Poll::Ready(Ok(()))
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        ready!(<Encoder<W> as Sink<T>>::poll_flush(self.as_mut(), cx))?;

        let mut this = self.project();
        ready!(this.inner.as_mut().poll_close(cx))?;

        Poll::Ready(Ok(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_util::{io::Cursor, SinkExt as _};

    #[test]
    fn test_encoder() {
        futures_executor::block_on(async {
            {
                let w: Cursor<Vec<u8>> = Cursor::new(vec![]);
            }
        })
    }
}
