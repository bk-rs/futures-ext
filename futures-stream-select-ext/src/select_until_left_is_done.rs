use core::{
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{
    stream::{Abortable, FusedStream, PollNext},
    Stream,
};
use pin_project_lite::pin_project;

use crate::select_until_left_is_done_with_strategy::{
    select_until_left_is_done_with_strategy, SelectUntilLeftIsDoneWithStrategy,
};

//
pin_project! {
    /// Stream for the [`select_until_left_is_done()`] function. See function docs for details.
    #[must_use = "streams do nothing unless polled"]
    pub struct SelectUntilLeftIsDone<St1, St2> {
        #[pin]
        inner: SelectUntilLeftIsDoneWithStrategy<St1, St2, fn(&mut PollNext) -> PollNext, PollNext>,
    }
}

//
pub fn select_until_left_is_done<St1, St2>(
    stream1: St1,
    stream2: St2,
) -> SelectUntilLeftIsDone<St1, St2>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
{
    fn round_robin(last: &mut PollNext) -> PollNext {
        last.toggle()
    }

    SelectUntilLeftIsDone {
        inner: select_until_left_is_done_with_strategy(stream1, stream2, round_robin),
    }
}

//
impl<St1, St2> SelectUntilLeftIsDone<St1, St2> {
    pub fn get_ref(&self) -> (&St1, &Abortable<St2>) {
        self.inner.get_ref()
    }

    pub fn get_mut(&mut self) -> (&mut St1, &mut Abortable<St2>) {
        self.inner.get_mut()
    }

    pub fn get_pin_mut(self: Pin<&mut Self>) -> (Pin<&mut St1>, Pin<&mut Abortable<St2>>) {
        let this = self.project();
        this.inner.get_pin_mut()
    }

    pub fn into_inner(self) -> (St1, Abortable<St2>) {
        self.inner.into_inner()
    }
}

//
impl<St1, St2> FusedStream for SelectUntilLeftIsDone<St1, St2>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

//
impl<St1, St2> Stream for SelectUntilLeftIsDone<St1, St2>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
{
    type Item = St1::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<St1::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}

#[cfg(feature = "alloc")]
#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{vec, vec::Vec};

    use futures_util::{stream, StreamExt as _};

    #[test]
    fn test() {
        futures_executor::block_on(async {
            for (range, ret) in vec![
                (1..=1, vec![1, 0]),
                (1..=2, vec![1, 0, 2, 0]),
                (1..=3, vec![1, 0, 2, 0, 3, 0]),
                (1..=4, vec![1, 0, 2, 0, 3, 0, 4, 0]),
                (1..=5, vec![1, 0, 2, 0, 3, 0, 4, 0, 5, 0]),
            ] {
                let st1 = stream::iter(range).boxed();
                let st2 = stream::repeat(0);

                let st = select_until_left_is_done(st1, st2);

                assert_eq!(st.collect::<Vec<_>>().await, ret);
            }
        })
    }
}
