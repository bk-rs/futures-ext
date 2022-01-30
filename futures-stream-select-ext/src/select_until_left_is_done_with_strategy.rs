use core::{
    fmt,
    pin::Pin,
    task::{Context, Poll},
};

use futures_util::{
    stream::{
        abortable, select_with_strategy, AbortHandle, Abortable, FusedStream, PollNext,
        SelectWithStrategy,
    },
    Stream,
};
use pin_project_lite::pin_project;

//
type Inner<St1, St2, State> =
    SelectWithStrategy<St1, Abortable<St2>, fn(&mut State) -> PollNext, State>;

pin_project! {
    /// Stream for the [`select_until_left_is_done_with_strategy()`] function. See function docs for details.
    #[must_use = "streams do nothing unless polled"]
    pub struct SelectUntilLeftIsDoneWithStrategy<St1, St2, Clos, State> {
        #[pin]
        inner: Inner<St1, St2, State>,
        abort_handle: AbortHandle,
        state: State,
        clos: Clos,
    }
}

//
pub fn select_until_left_is_done_with_strategy<St1, St2, Clos, State>(
    stream1: St1,
    stream2: St2,
    which: Clos,
) -> SelectUntilLeftIsDoneWithStrategy<St1, St2, Clos, State>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
    Clos: FnMut(&mut State) -> PollNext,
    State: Default,
{
    let (stream2, abort_handle) = abortable(stream2);

    SelectUntilLeftIsDoneWithStrategy {
        inner: select_with_strategy(stream1, stream2, |_| unreachable!()),
        abort_handle,
        state: Default::default(),
        clos: which,
    }
}

//
impl<St1, St2, Clos, State> SelectUntilLeftIsDoneWithStrategy<St1, St2, Clos, State> {
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
impl<St1, St2, Clos, State> FusedStream for SelectUntilLeftIsDoneWithStrategy<St1, St2, Clos, State>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
    Clos: FnMut(&mut State) -> PollNext,
{
    fn is_terminated(&self) -> bool {
        self.inner.is_terminated()
    }
}

//
impl<St1, St2, Clos, State> Stream for SelectUntilLeftIsDoneWithStrategy<St1, St2, Clos, State>
where
    St1: Stream,
    St2: Stream<Item = St1::Item>,
    Clos: FnMut(&mut State) -> PollNext,
{
    type Item = St1::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<St1::Item>> {
        let this = self.project();
        let (left, right) = this.inner.get_pin_mut();

        match (this.clos)(this.state) {
            PollNext::Left => {
                let left_done = match left.poll_next(cx) {
                    Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                    Poll::Ready(None) => {
                        this.abort_handle.abort();
                        true
                    }
                    Poll::Pending => false,
                };

                match right.poll_next(cx) {
                    Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
                    Poll::Ready(None) if left_done => Poll::Ready(None),
                    Poll::Ready(None) | Poll::Pending => Poll::Pending,
                }
            }
            PollNext::Right => {
                let right_done = match right.poll_next(cx) {
                    Poll::Ready(Some(item)) => return Poll::Ready(Some(item)),
                    Poll::Ready(None) => true,
                    Poll::Pending => false,
                };

                match left.poll_next(cx) {
                    Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
                    Poll::Ready(None) if right_done => Poll::Ready(None),
                    Poll::Ready(None) => {
                        this.abort_handle.abort();
                        Poll::Pending
                    }
                    Poll::Pending => Poll::Pending,
                }
            }
        }
    }
}

//
impl<St1, St2, Clos, State> fmt::Debug for SelectUntilLeftIsDoneWithStrategy<St1, St2, Clos, State>
where
    St1: fmt::Debug,
    St2: fmt::Debug,
    State: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (stream1, stream2) = self.get_ref();

        f.debug_struct("SelectUntilLeftIsDoneWithStrategy")
            .field("stream1", &stream1)
            .field("stream2", &stream2)
            .field("state", &self.state)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::{vec, vec::Vec};
    use core::time::Duration;

    use futures_util::{stream, StreamExt as _};

    fn round_robin(last: &mut PollNext) -> PollNext {
        last.toggle()
    }

    fn right_right_left(i: &mut usize) -> PollNext {
        let poll_next = if *i % 3 == 2 {
            PollNext::Left
        } else {
            PollNext::Right
        };

        *i += 1;
        poll_next
    }

    #[test]
    fn test_with_round_robin() {
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

                let st = select_until_left_is_done_with_strategy(st1, st2, round_robin);

                assert_eq!(st.collect::<Vec<_>>().await, ret);
            }
        })
    }

    #[test]
    fn test_with_right_right_left() {
        futures_executor::block_on(async {
            for (range, ret) in vec![
                (1..=1, vec![0, 0, 1, 0, 0]),
                (1..=2, vec![0, 0, 1, 0, 0, 2, 0, 0]),
                (1..=3, vec![0, 0, 1, 0, 0, 2, 0, 0, 3, 0, 0]),
                (1..=4, vec![0, 0, 1, 0, 0, 2, 0, 0, 3, 0, 0, 4, 0, 0]),
                (
                    1..=5,
                    vec![0, 0, 1, 0, 0, 2, 0, 0, 3, 0, 0, 4, 0, 0, 5, 0, 0],
                ),
            ] {
                let st1 = stream::iter(range).boxed();
                let st2 = stream::repeat(0);

                let st = select_until_left_is_done_with_strategy(st1, st2, right_right_left);

                assert_eq!(st.collect::<Vec<_>>().await, ret);
            }
        })
    }

    #[test]
    fn test_with_round_robin_and_right_long_sleep() {
        futures_executor::block_on(async {
            for (range, ret) in vec![
                (1..=1, vec![1]),
                (1..=2, vec![1, 2]),
                (1..=3, vec![1, 2, 3]),
                (1..=4, vec![1, 2, 3, 4]),
                (1..=5, vec![1, 2, 3, 4, 5]),
            ] {
                let st1 = stream::iter(range).boxed();
                let st2 = stream::repeat(0)
                    .then(|n| async move {
                        async_timer::interval(Duration::from_secs(5)).wait().await;
                        n
                    })
                    .boxed();

                let st = select_until_left_is_done_with_strategy(st1, st2, round_robin);

                #[cfg(feature = "std")]
                let now = std::time::Instant::now();

                assert_eq!(st.collect::<Vec<_>>().await, ret);

                #[cfg(feature = "std")]
                assert!(now.elapsed() < Duration::from_secs(1));
            }
        })
    }

    #[test]
    fn test_with_round_robin_and_both_sleep() {
        futures_executor::block_on(async {
            for (range, ret_vec) in vec![
                (1..=1, vec![vec![1]]),
                (1..=2, vec![vec![1, 0, 2]]),
                (1..=3, vec![vec![1, 0, 2, 3]]),
                (1..=4, vec![vec![1, 0, 2, 3, 0, 4]]),
                (1..=5, vec![vec![1, 0, 2, 3, 0, 4, 0, 5]]),
            ] {
                let st1 = stream::iter(range)
                    .then(|n| async move {
                        async_timer::interval(Duration::from_millis(100))
                            .wait()
                            .await;
                        n
                    })
                    .boxed();
                let st2 = stream::repeat(0)
                    .then(|n| async move {
                        async_timer::interval(Duration::from_millis(160))
                            .wait()
                            .await;
                        n
                    })
                    .boxed();

                let st = select_until_left_is_done_with_strategy(st1, st2, round_robin);

                #[cfg(feature = "std")]
                let now = std::time::Instant::now();

                let ret = st.collect::<Vec<_>>().await;
                #[cfg(feature = "std")]
                println!("ret {:?}", ret);
                assert!(ret_vec.contains(&ret));

                #[cfg(feature = "std")]
                assert!(now.elapsed() < Duration::from_secs(1));
            }
        })
    }

    #[test]
    fn test_with_round_robin_and_both_sleep_2() {
        futures_executor::block_on(async {
            for (range, ret_vec) in vec![
                (1..=1, vec![vec![0, 1]]),
                (1..=2, vec![vec![0, 1, 0, 2]]),
                (1..=3, vec![vec![0, 1, 0, 2, 0, 0, 3]]),
                (1..=4, vec![vec![0, 1, 0, 2, 0, 0, 3, 0, 4]]),
                (
                    1..=5,
                    vec![
                        vec![0, 1, 0, 2, 0, 0, 3, 0, 4, 0, 0, 5],
                        vec![0, 1, 0, 2, 0, 0, 3, 0, 4, 0, 5],
                    ],
                ),
            ] {
                let st1 = stream::iter(range)
                    .then(|n| async move {
                        async_timer::interval(Duration::from_millis(140))
                            .wait()
                            .await;
                        n
                    })
                    .boxed();
                let st2 = stream::repeat(0)
                    .then(|n| async move {
                        async_timer::interval(Duration::from_millis(100))
                            .wait()
                            .await;
                        n
                    })
                    .boxed();

                let st = select_until_left_is_done_with_strategy(st1, st2, round_robin);

                #[cfg(feature = "std")]
                let now = std::time::Instant::now();

                let ret = st.collect::<Vec<_>>().await;
                #[cfg(feature = "std")]
                println!("ret {:?}", ret);
                assert!(ret_vec.contains(&ret));

                #[cfg(feature = "std")]
                assert!(now.elapsed() < Duration::from_secs(1));
            }
        })
    }

    #[test]
    fn test_with_right_right_left_and_both_sleep() {
        futures_executor::block_on(async {
            for (range, ret_vec) in vec![
                (1..=1, vec![vec![0, 1]]),
                (1..=2, vec![vec![0, 1, 0, 0, 2]]),
                (1..=3, vec![vec![0, 1, 0, 0, 2, 3]]),
                (1..=4, vec![vec![0, 1, 0, 0, 2, 3, 4]]),
                (1..=5, vec![vec![0, 1, 0, 0, 2, 3, 4, 5]]),
            ] {
                let st1 = stream::iter(range)
                    .then(|n| async move {
                        async_timer::interval(Duration::from_millis(60))
                            .wait()
                            .await;
                        n
                    })
                    .boxed();
                let st2 = stream::iter(vec![0, 0, 0])
                    .then(|n| async move {
                        async_timer::interval(Duration::from_millis(35))
                            .wait()
                            .await;
                        n
                    })
                    .boxed();

                let st = select_until_left_is_done_with_strategy(st1, st2, right_right_left);

                #[cfg(feature = "std")]
                let now = std::time::Instant::now();

                let ret = st.collect::<Vec<_>>().await;
                #[cfg(feature = "std")]
                println!("ret {:?}", ret);
                assert!(ret_vec.contains(&ret));

                #[cfg(feature = "std")]
                assert!(now.elapsed() < Duration::from_secs(1));
            }
        })
    }
}
