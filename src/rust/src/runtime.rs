use std::{
    any::Any,
    cell::RefCell,
    future::Future,
    marker::PhantomData,
    mem::ManuallyDrop,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::callbacks::create_callback;
use crate::invoke::Js;

thread_local! {
    static STATE_MAP: RefCell<Vec<Box<dyn Any>>> = Default::default(); // Cast: RuntimeState<T>
}

enum RuntimeState<T> { Init, Pending(Waker), Competed(T) }

pub struct RuntimeFuture<T> { pub id: usize, phantom: PhantomData<T>, }
pub struct Runtime<T> { phantom: PhantomData<T> }

type FutureRc<T> = Rc::<RefCell<Pin<Box<dyn Future<Output = T>>>>>;

impl<T: Clone + 'static> Future for RuntimeFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        STATE_MAP.with_borrow_mut(|s| {
            let future = s[self.id].downcast_mut::<RuntimeState<T>>().unwrap();
            match future {
                RuntimeState::Competed(result) => {
                    let poll = Poll::Ready(result.to_owned());
                    s.remove(self.id);
                    poll
                },
                _ => {
                    *future = RuntimeState::Pending(cx.waker().to_owned());
                    Poll::Pending
                }
            }
        })
    }
}

impl <T: 'static> RuntimeFuture<T> {
    pub fn new() -> Self {
        STATE_MAP.with_borrow_mut(|s| {
            s.push(Box::new(RuntimeState::<T>::Init));
            Self { id: s.len() - 1, phantom: PhantomData::default() }
        })
    }

    // https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
    pub fn wake(future_id: usize, result: T) {
        STATE_MAP.with_borrow_mut(|s| {
            let future = s[future_id].downcast_mut::<RuntimeState<T>>().unwrap();

            if let RuntimeState::Pending(ref mut waker) = future { waker.to_owned().wake(); }
            *future = RuntimeState::Competed(result);
        });
    }
}

impl<T: 'static> Runtime<T> {

    fn poll(future_rc: &FutureRc<T>) {
        let waker = Self::waker(&future_rc);
        let waker_forget = ManuallyDrop::new(waker);
        let context = &mut Context::from_waker(&waker_forget);
        let _poll = future_rc.borrow_mut().as_mut().poll(context);
    }

    fn waker(future_rc: &FutureRc::<T>) -> Waker {

        fn clone_fn<T: 'static>(ptr: *const ()) -> RawWaker {
            let _future = unsafe { FutureRc::<T>::from_raw(ptr as *const _) };
            let _ = ManuallyDrop::new(_future).clone();
            RawWaker::new(ptr, waker_vtable::<T>())
        }
        fn wake_fn<T: 'static>(ptr: *const ()) {
            let _future = unsafe { FutureRc::<T>::from_raw(ptr as *const _) };
            let function_ref = create_callback(move |_| { Runtime::poll(&_future); });
            Js::invoke("window.setTimeout({},0)", &[function_ref.into()]);
        }
        fn drop_fn<T>(ptr: *const ()) {
            let _future = unsafe { FutureRc::<T>::from_raw(ptr as *const _) };
            drop(_future);
        }
        fn waker_vtable<T: 'static>() -> &'static RawWakerVTable {
            &RawWakerVTable::new(clone_fn::<T>, wake_fn::<T>, wake_fn::<T>, drop_fn::<T>)
        }
        let waker = RawWaker::new(&**future_rc as *const _ as *const (), waker_vtable::<T>());
        unsafe { Waker::from_raw(waker) }
    }

    pub fn block_on(future: impl Future<Output = T> + 'static) {
        Self::poll(&Rc::new(RefCell::new(Box::pin(future))));
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_await() {

        // create future
        let future = RuntimeFuture::new();
        assert_eq!(future.id, 0);

        STATE_MAP.with_borrow_mut(|s| {
            let state = s[future.id].downcast_mut::<RuntimeState<bool>>().unwrap();
            assert_eq!(matches!(state, RuntimeState::Init), true);
        });

        // wake future
        RuntimeFuture::wake(future.id, true);
        STATE_MAP.with_borrow_mut(|s| {
            let state = s[future.id].downcast_mut::<RuntimeState<bool>>().unwrap();
            assert_eq!(matches!(state, RuntimeState::Competed(true)), true);
        });

        // block on future
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        Runtime::block_on(async move { *has_run_clone.borrow_mut() = future.await; });
        assert_eq!(*has_run.borrow(), true);
    }

}
