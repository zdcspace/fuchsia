//! Definition of the TryMaybeDone combinator

use core::mem;
use core::pin::Pin;
use futures_core::future::{FusedFuture, Future, TryFuture};
use futures_core::task::{Context, Poll};
use pin_project::{pin_project, project};

/// A future that may have completed with an error.
///
/// This is created by the [`try_maybe_done()`] function.
#[pin_project]
#[derive(Debug)]
pub enum TryMaybeDone<Fut: TryFuture> {
    /// A not-yet-completed future
    Future(#[pin] Fut),
    /// The output of the completed future
    Done(Fut::Ok),
    /// The empty variant after the result of a [`TryMaybeDone`] has been
    /// taken using the [`take_output`](TryMaybeDone::take_output) method,
    /// or if the future returned an error.
    Gone,
}

/// Wraps a future into a `TryMaybeDone`
pub fn try_maybe_done<Fut: TryFuture>(future: Fut) -> TryMaybeDone<Fut> {
    TryMaybeDone::Future(future)
}

impl<Fut: TryFuture> TryMaybeDone<Fut> {
    /// Returns an [`Option`] containing a mutable reference to the output of the future.
    /// The output of this method will be [`Some`] if and only if the inner
    /// future has completed successfully and [`take_output`](TryMaybeDone::take_output)
    /// has not yet been called.
    #[project]
    #[inline]
    pub fn output_mut(self: Pin<&mut Self>) -> Option<&mut Fut::Ok> {
        #[project]
        match self.project() {
            TryMaybeDone::Done(res) => Some(res),
            _ => None,
        }
    }

    /// Attempt to take the output of a `TryMaybeDone` without driving it
    /// towards completion.
    #[inline]
    pub fn take_output(self: Pin<&mut Self>) -> Option<Fut::Ok> {
        // Safety: we return immediately unless we are in the `Done`
        // state, which does not have any pinning guarantees to uphold.
        //
        // Hopefully `pin_project` will support this safely soon:
        // https://github.com/taiki-e/pin-project/issues/184
        unsafe {
            let this = self.get_unchecked_mut();
            match this {
                TryMaybeDone::Done(_) => {},
                TryMaybeDone::Future(_) | TryMaybeDone::Gone => return None,
            };
            if let TryMaybeDone::Done(output) = mem::replace(this, TryMaybeDone::Gone) {
                Some(output)
            } else {
                unreachable!()
            }
        }
    }
}

impl<Fut: TryFuture> FusedFuture for TryMaybeDone<Fut> {
    fn is_terminated(&self) -> bool {
        match self {
            TryMaybeDone::Future(_) => false,
            TryMaybeDone::Done(_) | TryMaybeDone::Gone => true,
        }
    }
}

impl<Fut: TryFuture> Future for TryMaybeDone<Fut> {
    type Output = Result<(), Fut::Error>;

    #[project]
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        #[project]
        match self.as_mut().project() {
            TryMaybeDone::Future(f) => {
                match ready!(f.try_poll(cx)) {
                    Ok(res) => self.set(TryMaybeDone::Done(res)),
                    Err(e) => {
                        self.set(TryMaybeDone::Gone);
                        return Poll::Ready(Err(e));
                    }
                }
            },
            TryMaybeDone::Done(_) => {},
            TryMaybeDone::Gone => panic!("TryMaybeDone polled after value taken"),
        }
        Poll::Ready(Ok(()))
    }
}
