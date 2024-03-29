// Ref https://github.com/tokio-rs/tokio/blob/tokio-util-0.7.7/tokio-util/src/io/reader_stream.rs

use core::pin::Pin;
use std::io::Error as IoError;

use futures_util::{stream::unfold, AsyncRead, AsyncReadExt as _, Stream};

//
const DEFAULT_CAPACITY: usize = 4096;

//
pub fn reader<R: AsyncRead + Unpin + Send + 'static>(
    reader: R,
) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, IoError>> + Send + 'static>> {
    reader_with_capacity(reader, DEFAULT_CAPACITY)
}

pub fn reader_with_capacity<R: AsyncRead + Unpin + Send + 'static>(
    reader: R,
    capacity: usize,
) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, IoError>> + Send + 'static>> {
    let buf = vec![0; capacity];
    let st = unfold((reader, buf), |(mut reader, mut buf)| async {
        match reader.read(&mut buf).await {
            Ok(n) if n == 0 => None,
            Ok(n) => Some((Ok(buf[..n].to_vec()), (reader, buf))),
            Err(err) => Some((Err(err), (reader, buf))),
        }
    });
    Box::pin(st)
}

pub fn reader_ref<'a, R: AsyncRead + Unpin + Send>(
    reader: &'a mut R,
) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, IoError>> + Send + 'a>> {
    reader_ref_with_capacity(reader, DEFAULT_CAPACITY)
}

pub fn reader_ref_with_capacity<'a, R: AsyncRead + Unpin + Send>(
    reader: &'a mut R,
    capacity: usize,
) -> Pin<Box<dyn Stream<Item = Result<Vec<u8>, IoError>> + Send + 'a>> {
    let buf = vec![0; capacity];
    let st = unfold((reader, buf), |(reader, mut buf)| async {
        match reader.read(&mut buf).await {
            Ok(n) if n == 0 => None,
            Ok(n) => Some((Ok(buf[..n].to_vec()), (reader, buf))),
            Err(err) => Some((Err(err), (reader, buf))),
        }
    });
    Box::pin(st)
}

#[cfg(test)]
mod tests {
    use super::*;

    use futures_util::{io::Cursor, StreamExt as _};

    #[test]
    fn simple() {
        futures_executor::block_on(async {
            {
                let r = Cursor::new(b"1234567890");

                let mut st = reader_with_capacity(r, 3);
                let mut n = 0;
                while let Some(ret) = st.next().await {
                    match ret {
                        Ok(bytes) => {
                            n += 1;
                            match n {
                                1 => assert_eq!(bytes, b"123"),
                                2 => assert_eq!(bytes, b"456"),
                                3 => assert_eq!(bytes, b"789"),
                                4 => assert_eq!(bytes, b"0"),
                                _ => unreachable!(),
                            }
                        }
                        Err(err) => panic!("{err:?}"),
                    }
                }
                assert_eq!(n, 4);
            }

            {
                let mut r = Cursor::new(b"1234567890");

                let mut st = reader_ref_with_capacity(&mut r, 3);
                let mut n = 0;
                while let Some(ret) = st.next().await {
                    match ret {
                        Ok(bytes) => {
                            n += 1;
                            match n {
                                1 => assert_eq!(bytes, b"123"),
                                2 => assert_eq!(bytes, b"456"),
                                3 => assert_eq!(bytes, b"789"),
                                4 => assert_eq!(bytes, b"0"),
                                _ => unreachable!(),
                            }
                        }
                        Err(err) => panic!("{err:?}"),
                    }
                }
                assert_eq!(n, 4);
            }
        })
    }
}
