#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
mod select_until_left_is_done;
#[cfg(feature = "alloc")]
mod select_until_left_is_done_with_strategy;

#[cfg(feature = "alloc")]
pub use self::select_until_left_is_done::{select_until_left_is_done, SelectUntilLeftIsDone};
#[cfg(feature = "alloc")]
pub use self::select_until_left_is_done_with_strategy::{
    select_until_left_is_done_with_strategy, SelectUntilLeftIsDoneWithStrategy,
};
