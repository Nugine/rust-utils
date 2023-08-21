use crate::waker::AtomicWaker;

use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::{Context, Poll, Waker};

#[repr(C)]
struct Inner {
    refcount: AtomicUsize,
    waker: AtomicWaker,
}

pub struct InnerPtr(NonNull<Inner>);

unsafe impl Send for InnerPtr {}
unsafe impl Sync for InnerPtr {}

impl InnerPtr {
    pub fn new() -> Self {
        let p = Box::into_raw(Box::new(Inner {
            refcount: AtomicUsize::new(1),
            waker: AtomicWaker::new(),
        }));
        unsafe { Self(NonNull::new_unchecked(p)) }
    }

    #[inline(always)]
    fn deref(&self) -> &Inner {
        unsafe { self.0.as_ref() }
    }

    pub fn count(&self) -> usize {
        self.deref().refcount.load(Ordering::SeqCst) - 1
    }

    pub fn register_waker(&self, waker: &Waker) {
        self.deref().waker.register(waker);
    }

    #[allow(clippy::needless_pass_by_ref_mut)]
    pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<()> {
        if self.count() == 0 {
            return Poll::Ready(());
        }
        self.register_waker(cx.waker());
        if self.count() == 0 {
            return Poll::Ready(());
        }
        Poll::Pending
    }
}

impl Clone for InnerPtr {
    fn clone(&self) -> Self {
        let old_refcount = self.deref().refcount.fetch_add(1, Ordering::Relaxed);
        #[cfg(not(target_pointer_width = "64"))]
        {
            const MAX_REFCOUNT: usize = (isize::MAX) as usize;
            if old_refcount > MAX_REFCOUNT {
                std::process::abort();
            }
        }
        #[cfg(target_pointer_width = "64")]
        {
            let _ = old_refcount;
        }
        Self(self.0)
    }
}

impl Drop for InnerPtr {
    fn drop(&mut self) {
        #[inline(never)]
        unsafe fn drop_slow(this: *mut Inner, old_refcount: usize) {
            match old_refcount {
                2 => (*this).waker.wake(),
                1 => drop(Box::from_raw(this)),
                _ => {}
            }
        }

        let old_refcount = self.deref().refcount.fetch_sub(1, Ordering::Release);
        if old_refcount > 2 {
            return;
        }
        self.deref().refcount.load(Ordering::Acquire);
        unsafe { drop_slow(self.0.as_ptr(), old_refcount) }
    }
}
