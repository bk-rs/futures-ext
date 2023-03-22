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
//
//
pin_project! {
    #[derive(Debug)]
    pub struct Decoder<R> {
        #[pin]
        inner: R,
        buf: Vec<u8>,
        n_read: usize,
        state: DecodeState,
    }
}

impl<R: AsyncRead> Decoder<R> {
    pub fn new(inner: R) -> Self {
        Self::with_capacity(1024, inner)
    }

    pub fn with_capacity(cap: usize, inner: R) -> Self {
        Self {
            inner,
            buf: vec![0; cap],
            n_read: 0,
            state: DecodeState::Head,
        }
    }

    pub fn get_ref(&self) -> &R {
        &self.inner
    }

    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }

    pub fn into_inner(self) -> R {
        self.inner
    }
}

#[derive(Debug, Clone, Copy)]
enum DecodeState {
    Head,
    Data(usize),
}

impl<R: AsyncRead> Stream for Decoder<R> {
    type Item = Result<Vec<u8>, IoError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        let field_len = core::mem::size_of::<u64>();

        loop {
            match *this.state {
                DecodeState::Head => {
                    if *this.n_read >= field_len {
                        let data_len =
                            u64::from_be_bytes(this.buf[..field_len].try_into().expect("Never"));

                        if this.buf.len() < data_len as usize {
                            this.buf.resize(data_len as usize, 0);
                        }
                        this.buf.rotate_left(field_len);
                        *this.n_read -= field_len;

                        *this.state = DecodeState::Data(data_len as usize);
                        continue;
                    }
                }
                DecodeState::Data(data_len) => {
                    if *this.n_read >= data_len {
                        let data = this.buf[..data_len].to_vec();

                        this.buf.rotate_left(data_len);
                        *this.n_read -= data_len;

                        *this.state = DecodeState::Head;

                        return Poll::Ready(Some(Ok(data)));
                    }
                }
            }

            match ready!(this
                .inner
                .as_mut()
                .poll_read(cx, &mut this.buf[*this.n_read..]))
            {
                Ok(n) => {
                    if n == 0 {
                        match *this.state {
                            DecodeState::Head => {
                                if *this.n_read == 0 {
                                    return Poll::Ready(None);
                                } else {
                                    return Poll::Ready(Some(Err(IoError::new(
                                        IoErrorKind::Other,
                                        format!("need more head, n:{}", field_len - *this.n_read),
                                    ))));
                                }
                            }
                            DecodeState::Data(data_len) => {
                                if *this.n_read == 0 {
                                    return Poll::Ready(Some(Err(IoError::new(
                                        IoErrorKind::Other,
                                        "no data".to_string(),
                                    ))));
                                } else {
                                    return Poll::Ready(Some(Err(IoError::new(
                                        IoErrorKind::Other,
                                        format!("need more data, n:{}", data_len - *this.n_read),
                                    ))));
                                }
                            }
                        }
                    }
                    *this.n_read += n;
                }
                Err(err) => {
                    //
                    return Poll::Ready(Some(Err(err)));
                }
            }
        }
    }
}

//
//
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
        Self::with_capacity(1024, inner)
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
        let data_len = data.len() as u64;

        this.buf.extend_from_slice(data_len.to_be_bytes().as_ref());
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
        this.buf.clear();

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

    use futures_util::{io::Cursor, SinkExt as _, StreamExt as _};

    #[test]
    fn simple() -> Result<(), Box<dyn std::error::Error>> {
        futures_executor::block_on(async {
            let cursor: Cursor<Vec<u8>> = Cursor::new(vec![]);

            let mut decoder = Decoder::new(cursor);
            assert!(decoder.next().await.is_none());

            let cursor = decoder.into_inner();

            let mut encoder = Encoder::new(cursor);
            encoder.send(&"abc").await?;
            encoder.send(&"12").await?;
            encoder.send(&[]).await?;

            let mut cursor = encoder.into_inner();
            cursor.set_position(0);

            let mut decoder = Decoder::new(cursor);
            assert_eq!(
                decoder.next().await.ok_or("decoder.next() is_none")??,
                b"abc"
            );
            assert_eq!(
                decoder.next().await.ok_or("decoder.next() is_none")??,
                b"12"
            );
            assert_eq!(decoder.next().await.ok_or("decoder.next() is_none")??, b"");
            assert!(decoder.next().await.is_none());

            Ok(())
        })
    }

    #[test]
    fn test_decoder() -> Result<(), Box<dyn std::error::Error>> {
        futures_executor::block_on(async {
            let mut r: Cursor<Vec<u8>> = Cursor::new(vec![
                0, 0, 0, 0, 0, 0, 0, 3, //
                97, 98, 99, //
            ]);
            r.set_position(0);
            let mut decoder = Decoder::new(r);
            assert_eq!(
                decoder.next().await.ok_or("decoder.next() is_none")??,
                b"abc"
            );
            assert!(decoder.next().await.is_none());

            let mut r: Cursor<Vec<u8>> = Cursor::new(vec![
                0, 0, 0, 0, 0, 0, 0, 3, //
                97, 98, 99, //
                0, 0, 0,
            ]);
            r.set_position(0);
            let mut decoder = Decoder::new(r);
            assert_eq!(
                decoder.next().await.ok_or("decoder.next() is_none")??,
                b"abc"
            );
            match decoder.next().await {
                Some(Err(err)) => {
                    assert_eq!(err.kind(), IoErrorKind::Other);
                    assert!(err.to_string().contains("need more head, n:5"));
                }
                x => panic!("{x:?}"),
            };

            let mut r: Cursor<Vec<u8>> = Cursor::new(vec![
                0, 0, 0, 0, 0, 0, 0, 2, //
                1, 2, //
                0, 0, 0, 0, 0, 0, 0, 1, //
                3, //
                0, 0, 0, 0, 0, 0, 0, 3, //
                4, 5, 6, //
            ]);
            r.set_position(0);
            let mut decoder = Decoder::new(r);
            assert_eq!(
                decoder.next().await.ok_or("decoder.next() is_none")??,
                &[1, 2]
            );
            assert_eq!(decoder.next().await.ok_or("decoder.next() is_none")??, &[3]);
            assert_eq!(
                decoder.next().await.ok_or("decoder.next() is_none")??,
                &[4, 5, 6]
            );
            assert!(decoder.next().await.is_none());

            Ok(())
        })
    }

    #[test]
    fn test_encoder() -> Result<(), Box<dyn std::error::Error>> {
        futures_executor::block_on(async {
            let w: Cursor<Vec<u8>> = Cursor::new(vec![]);
            let mut encoder = Encoder::new(w);
            encoder.send(&"abc").await?;
            assert_eq!(
                encoder.into_inner().get_ref(),
                &[
                    0, 0, 0, 0, 0, 0, 0, 3, //
                    97, 98, 99, //
                ]
            );

            let w: Cursor<Vec<u8>> = Cursor::new(vec![]);
            let mut encoder = Encoder::new(w);
            encoder.send(&[1, 2]).await?;
            encoder.send(&[3]).await?;
            encoder.send(vec![4, 5, 6]).await?;
            assert_eq!(
                encoder.into_inner().get_ref(),
                &[
                    0, 0, 0, 0, 0, 0, 0, 2, //
                    1, 2, //
                    0, 0, 0, 0, 0, 0, 0, 1, //
                    3, //
                    0, 0, 0, 0, 0, 0, 0, 3, //
                    4, 5, 6, //
                ]
            );

            Ok(())
        })
    }
}
