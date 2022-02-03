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
    #[derive(Debug)]
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

    #[tokio::test]
    async fn test_with_both_sleep() {
        for (range, ret_vec) in vec![
            (1..=1, vec![vec![0, 1]]),
            (1..=2, vec![vec![0, 1, 0, 2]]),
            (1..=3, vec![vec![0, 1, 0, 2, 0, 3]]),
            (1..=4, vec![vec![0, 1, 0, 2, 0, 3, 0, 4]]),
            (1..=5, vec![vec![0, 1, 0, 2, 0, 3, 0, 4, 0, 5]]),
        ] {
            let st1 = stream::iter(range)
                .then(|n| async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(55)).await;
                    n
                })
                .boxed();
            let st2 =
                stream::repeat_with(|| tokio::time::sleep(tokio::time::Duration::from_millis(50)))
                    .then(|sleep| async move {
                        sleep.await;
                        0
                    })
                    .boxed();

            let st = select_until_left_is_done(st1, st2);

            #[cfg(feature = "std")]
            let now = std::time::Instant::now();

            let ret = st.collect::<Vec<_>>().await;
            #[cfg(feature = "std")]
            println!("ret {:?}", ret);
            assert!(ret_vec.contains(&ret));

            #[cfg(feature = "std")]
            assert!(now.elapsed() < core::time::Duration::from_secs(1));
        }
    }

    #[tokio::test]
    async fn test_with_both_sleep_2() {
        for (range, ret_vec) in vec![
            (1..=1, vec![vec![0, 1]]),
            (1..=2, vec![vec![0, 1, 0, 2]]),
            (1..=3, vec![vec![0, 1, 0, 2, 0, 3]]),
            (1..=4, vec![vec![0, 1, 0, 2, 0, 3, 0, 4]]),
            (1..=5, vec![vec![0, 1, 0, 2, 0, 3, 0, 4, 0, 5]]),
        ] {
            let st1 = stream::iter(range)
                .then(|n| async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(55)).await;
                    n
                })
                .boxed();
            let st2 = stream::unfold(
                (
                    0_usize,
                    tokio::time::interval(tokio::time::Duration::from_millis(50)),
                ),
                |(i, mut interval)| async move {
                    interval.reset();
                    interval.tick().await;
                    Some((0, (i + 1, interval)))
                },
            )
            .boxed();

            let st = select_until_left_is_done(st1, st2);

            #[cfg(feature = "std")]
            let now = std::time::Instant::now();

            let ret = st.collect::<Vec<_>>().await;
            #[cfg(feature = "std")]
            println!("ret {:?}", ret);
            assert!(ret_vec.contains(&ret));

            #[cfg(feature = "std")]
            assert!(now.elapsed() < core::time::Duration::from_secs(1));
        }
    }
}
