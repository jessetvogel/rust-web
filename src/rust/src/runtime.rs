use std::{
    cell::RefCell,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::callbacks::create_callback;
use crate::invoke::Js;

pub enum FutureState<T> { Init, Pending(Waker), Ready(T) }
pub struct FutureTask<T> { pub state: Rc<RefCell<FutureState<T>>> }

pub struct Runtime {}

type FutureRc<T> = Rc<RefCell<Pin<Box<dyn Future<Output = T>>>>>;

impl<T: Clone + 'static> Future for FutureTask<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        let mut future = self.state.borrow_mut();
        match &*future {
            FutureState::Ready(result) => {
                Poll::Ready(result.to_owned())
            },
            _ => {
                *future = FutureState::Pending(cx.waker().to_owned());
                Poll::Pending
            }
        }
    }
}

impl Runtime {

    fn poll<T: 'static>(future_rc: &FutureRc<T>) {
        let waker = Self::waker(&future_rc);
        let waker_forget = ManuallyDrop::new(waker);
        let context = &mut Context::from_waker(&waker_forget);
        let _poll = future_rc.borrow_mut().as_mut().poll(context);
    }

    // https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
    fn waker<T: 'static>(future_rc: &FutureRc::<T>) -> Waker {

        fn clone_fn<T: 'static>(ptr: *const ()) -> RawWaker {
            let future = unsafe { FutureRc::<T>::from_raw(ptr as *const _) };
            let _ = ManuallyDrop::new(future).clone();
            RawWaker::new(ptr, waker_vtable::<T>())
        }
        fn wake_fn<T: 'static>(ptr: *const ()) {
            let future = unsafe { FutureRc::<T>::from_raw(ptr as *const _) };
            let function_ref = create_callback(move |_| { Runtime::poll(&future); });
            Js::invoke("window.setTimeout({},0)", &[function_ref.into()]);
        }
        fn drop_fn<T>(ptr: *const ()) {
            let future = unsafe { FutureRc::<T>::from_raw(ptr as *const _) };
            drop(future);
        }
        fn waker_vtable<T: 'static>() -> &'static RawWakerVTable {
            &RawWakerVTable::new(clone_fn::<T>, wake_fn::<T>, wake_fn::<T>, drop_fn::<T>)
        }
        let waker = RawWaker::new(&**future_rc as *const _ as *const (), waker_vtable::<T>());
        unsafe { Waker::from_raw(waker) }
    }

    pub fn block_on<T: 'static>(future: impl Future<Output = T> + 'static) {
        Self::poll(&Rc::new(RefCell::new(Box::pin(future))));
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_await() {

        // create future
        let future = FutureTask { state: Rc::new(RefCell::new(FutureState::Init)) };
        let future_state = future.state.clone();
        assert_eq!(matches!(*future_state.borrow(), FutureState::Init), true);

        // set to ready
        *future_state.borrow_mut() = FutureState::Ready(true);
        assert_eq!(matches!(*future_state.borrow(), FutureState::Ready(true)), true);

        // block on future
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        Runtime::block_on(async move { *has_run_clone.borrow_mut() = future.await; });
        assert_eq!(*has_run.borrow(), true);
    }

}
