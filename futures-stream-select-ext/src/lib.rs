#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod select_until_left_is_done;
mod select_until_left_is_done_with_strategy;

pub use self::select_until_left_is_done::{select_until_left_is_done, SelectUntilLeftIsDone};
pub use self::select_until_left_is_done_with_strategy::{
    select_until_left_is_done_with_strategy, SelectUntilLeftIsDoneWithStrategy,
};
