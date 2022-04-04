use std::io::{BufRead, Read, Result, Seek, SeekFrom, Write};

use super::Either;

//
// Ref https://github.com/bluss/either/blob/1.6.1/src/lib.rs#L843
impl<A, B> Read for Either<A, B>
where
    A: Read,
    B: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self {
            Self::Left(x) => x.read(buf),
            Self::Right(x) => x.read(buf),
        }
    }

    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        match self {
            Self::Left(x) => x.read_to_end(buf),
            Self::Right(x) => x.read_to_end(buf),
        }
    }
}

// Ref https://github.com/bluss/either/blob/1.6.1/src/lib.rs#L877
impl<A, B> Write for Either<A, B>
where
    A: Write,
    B: Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            Self::Left(x) => x.write(buf),
            Self::Right(x) => x.write(buf),
        }
    }
    fn flush(&mut self) -> Result<()> {
        match self {
            Self::Left(x) => x.flush(),
            Self::Right(x) => x.flush(),
        }
    }
}

impl<A, B> Seek for Either<A, B>
where
    A: Seek,
    B: Seek,
{
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        match self {
            Self::Left(x) => x.seek(pos),
            Self::Right(x) => x.seek(pos),
        }
    }
}

// Ref https://github.com/bluss/either/blob/1.6.1/src/lib.rs#L859
impl<A, B> BufRead for Either<A, B>
where
    A: BufRead,
    B: BufRead,
{
    fn fill_buf(&mut self) -> Result<&[u8]> {
        match self {
            Self::Left(x) => x.fill_buf(),
            Self::Right(x) => x.fill_buf(),
        }
    }
    fn consume(&mut self, amt: usize) {
        match self {
            Self::Left(x) => x.consume(amt),
            Self::Right(x) => x.consume(amt),
        }
    }
}
