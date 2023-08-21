#![deny(
    clippy::all,
    // clippy::cargo,
    clippy::missing_inline_in_public_items,
    clippy::must_use_candidate
)]

mod waker {
    #[cfg(feature = "futures-util")]
    pub use futures_util::task::AtomicWaker;

    #[cfg(all(not(feature = "futures-util"), feature = "atomic-waker"))]
    pub use atomic_waker::AtomicWaker;

    #[cfg(all(not(feature = "atomic-waker"), not(feature = "futures-util")))]
    compile_error!("Please select an AtomicWaker implementation: futures-util or atomic-waker");
}

mod inner;
use self::inner::InnerPtr;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct WaitGroup(InnerPtr);

#[derive(Clone)]
pub struct Working(InnerPtr);

impl Working {
    #[inline]
    #[must_use]
    pub fn count(&self) -> usize {
        self.0.count()
    }
}

impl WaitGroup {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(InnerPtr::new())
    }

    #[inline]
    #[must_use]
    pub fn working(&self) -> Working {
        Working(self.0.clone())
    }

    #[inline]
    #[must_use]
    pub fn count(&self) -> usize {
        self.0.count()
    }

    #[inline]
    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.0.poll_wait(cx)
    }

    #[inline]
    #[must_use]
    pub fn wait(&self) -> WaitFuture<'_> {
        WaitFuture(self)
    }

    #[inline]
    #[must_use]
    pub fn wait_owned(self) -> WaitOwnedFuture {
        WaitOwnedFuture(self)
    }
}

impl Default for WaitGroup {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

pub struct WaitOwnedFuture(WaitGroup);

impl Future for WaitOwnedFuture {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_wait(cx)
    }
}

impl AsRef<WaitGroup> for WaitOwnedFuture {
    #[inline]
    fn as_ref(&self) -> &WaitGroup {
        &self.0
    }
}

pub struct WaitFuture<'a>(&'a WaitGroup);

impl Future for WaitFuture<'_> {
    type Output = ();

    #[inline]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.0.poll_wait(cx)
    }
}

impl AsRef<WaitGroup> for WaitFuture<'_> {
    #[inline]
    fn as_ref(&self) -> &WaitGroup {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::WaitGroup;

    use tokio::time::{sleep, Duration};

    #[test]
    fn simple() {
        let wg = WaitGroup::new();
        let n = 100;
        let working_vec = vec![wg.working(); n];
        assert_eq!(wg.count(), n);
        drop(wg);
        drop(working_vec);
    }

    #[tokio::test]
    async fn tokio_test() {
        let wg = WaitGroup::new();
        let n = 100;

        assert_eq!(wg.count(), 0);
        for _ in 0..n {
            let working = wg.working();
            tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                drop(working);
            });
        }
        assert_eq!(wg.count(), n);
        wg.wait().await;

        assert_eq!(wg.count(), 0);
        for _ in 0..n {
            let working = wg.working();
            tokio::spawn(async move {
                sleep(Duration::from_millis(50)).await;
                drop(working);
            });
        }
        assert_eq!(wg.count(), n);
        wg.wait_owned().await;
    }
}
