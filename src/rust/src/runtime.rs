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

pub enum FutureState<T> { Init, Pending(Waker), Competed(T) }
pub struct FutureTask<T> { phantom: PhantomData<T>, pub map: Rc<RefCell<Box<dyn Any>>> }

pub struct Runtime<T> { phantom: PhantomData<T> }

type FutureRc<T> = Rc<RefCell<Pin<Box<dyn Future<Output = T>>>>>;

impl<T: Clone + 'static> Future for FutureTask<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {

        let mut map = self.map.borrow_mut();
        let future = map.downcast_mut::<FutureState<T>>().unwrap();
        match future {
            FutureState::Competed(result) => {
                Poll::Ready(result.to_owned())
            },
            _ => {
                *future = FutureState::Pending(cx.waker().to_owned());
                Poll::Pending
            }
        }
    }
}

impl <T: 'static> FutureTask<T> {
    pub fn new() -> Self {
        Self { phantom: PhantomData::default(), map: Rc::new(RefCell::new(Box::new(FutureState::<T>::Init))) }
    }

    // https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
    pub fn wake(map: &Rc<RefCell<Box<dyn Any>>>, result: T) {
        let mut map = map.borrow_mut();
        let future = map.downcast_mut::<FutureState<T>>().unwrap();

        if let FutureState::Pending(ref mut waker) = future { waker.to_owned().wake(); }
        *future = FutureState::Competed(result);
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
        let future = FutureTask::new();
        let future_map = future.map.clone();
        let mut future_borrow = future_map.borrow_mut();
        let state = future_borrow.downcast_mut::<FutureState<bool>>().unwrap();
        assert_eq!(matches!(state, FutureState::Init), true);
        drop(future_borrow);

        // wake future
        let future_map = future.map.clone();
        FutureTask::wake(&future_map, true);
        let future_map = future.map.clone();
        let mut future_borrow = future_map.borrow_mut();
        let state = future_borrow.downcast_mut::<FutureState<bool>>().unwrap();
        assert_eq!(matches!(state, FutureState::Competed(true)), true);
        drop(future_borrow);

        // block on future
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        Runtime::block_on(async move { *has_run_clone.borrow_mut() = future.await; });
        assert_eq!(*has_run.borrow(), true);
    }

}
