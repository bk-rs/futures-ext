#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

use core::pin::Pin;

#[cfg(feature = "futures_io")]
mod impl_futures_io;
#[cfg(feature = "std")]
mod impl_std_io;

//
// Ref https://github.com/rust-lang/futures-rs/blob/0.3.21/futures-util/src/future/either.rs#L27
#[derive(Debug, Clone)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}

impl<A, B> Either<A, B> {
    #[cfg(feature = "futures-util")]
    pub fn into_futures_util_either(self) -> futures_util::future::Either<A, B> {
        match self {
            Either::Left(a) => futures_util::future::Either::Left(a),
            Either::Right(b) => futures_util::future::Either::Right(b),
        }
    }

    #[cfg(feature = "either")]
    pub fn into_either(self) -> either::Either<A, B> {
        match self {
            Either::Left(a) => either::Either::Left(a),
            Either::Right(b) => either::Either::Right(b),
        }
    }
}

// Ref https://github.com/rust-lang/futures-rs/blob/0.3.21/futures-util/src/future/either.rs#L35
impl<A, B> Either<A, B> {
    fn project(self: Pin<&mut Self>) -> Either<Pin<&mut A>, Pin<&mut B>> {
        unsafe {
            match self.get_unchecked_mut() {
                Either::Left(a) => Either::Left(Pin::new_unchecked(a)),
                Either::Right(b) => Either::Right(Pin::new_unchecked(b)),
            }
        }
    }
}
