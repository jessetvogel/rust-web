use std::{
    any::Any,
    cell::RefCell,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::callbacks::create_callback;
use crate::invoke::Js;

thread_local! {
    static STATE_MAP: RefCell<Vec<Box<dyn Any>>> = Default::default(); // Cast: Rc<RefCell<RuntimeState<T>>>
}

enum RuntimeState<T> { Pending(Option<Waker>), Competed(T) }

pub struct RuntimeFuture<T> { id: u32, state: Rc<RefCell<RuntimeState<T>>>, }
pub struct Runtime<T> { future: RefCell<Pin<Box<dyn Future<Output = T>>>>, }

impl<T: Clone> Future for RuntimeFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match *self.state.borrow_mut() {
            RuntimeState::Pending(ref mut waker) => {
                *waker = Some(cx.waker().to_owned());
                Poll::Pending
            },
            RuntimeState::Competed(ref result) => {
                Poll::Ready(result.to_owned())
            },
        }
    }
}

// https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
impl <T: 'static> RuntimeFuture<T> {
    pub fn new() -> Self {

        let state = RuntimeState::Pending(None);
        let state_arc = Rc::new(RefCell::new(state));
        let future_id = STATE_MAP.with_borrow_mut(|s| {
            s.push(Box::new(state_arc.clone()));
            s.len() - 1
        });

        Self { id: future_id as u32, state: state_arc }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn wake(future_id: u32, result: T) {
        STATE_MAP.with_borrow_mut(|s| {
            let future = s[future_id as usize].downcast_mut::<Rc<RefCell<RuntimeState<T>>>>().unwrap();

            if let RuntimeState::Pending(ref mut waker) = *future.borrow_mut() {
                if let Some(waker) = waker.as_mut() { waker.to_owned().wake(); }
            }
            *future.borrow_mut() = RuntimeState::Competed(result);

            s.remove(future_id as usize);
        });
    }
}

impl<T: 'static> Runtime<T> {

    fn poll(runtime: &Rc<Self>) {
        let waker = Self::waker(&runtime);
        let waker_forget = ManuallyDrop::new(waker);
        let context = &mut Context::from_waker(&waker_forget);
        let _poll = runtime.future.borrow_mut().as_mut().poll(context);
    }

    fn waker(runtime: &Rc<Self>) -> Waker {

        fn clone_fn<T: 'static>(ptr: *const ()) -> RawWaker {
            let _runtime = unsafe { Rc::<Runtime<T>>::from_raw(ptr as *const _) };
            let _ = ManuallyDrop::new(_runtime).clone();

            RawWaker::new(ptr, waker_vtable::<T>())
        }
        fn wake_fn<T: 'static>(ptr: *const ()) {
            let _runtime = unsafe { Rc::<Runtime<T>>::from_raw(ptr as *const _) };
            let function_ref = create_callback(move |_| { Runtime::poll(&_runtime); });
            Js::invoke("window.setTimeout({},0)", &[function_ref.into()]);
        }
        fn drop_fn<T>(ptr: *const ()) {
            let _runtime = unsafe { Rc::<Runtime<T>>::from_raw(ptr as *const _) };
            drop(_runtime);
        }
        fn waker_vtable<T: 'static>() -> &'static RawWakerVTable {
            &RawWakerVTable::new(clone_fn::<T>, wake_fn::<T>, wake_fn::<T>, drop_fn::<T>)
        }
        let ptr = &**runtime as *const _;
        let raw_waker = RawWaker::new(ptr as *const (), waker_vtable::<T>());
        unsafe { Waker::from_raw(raw_waker) }
    }

    pub fn block_on(future: impl Future<Output = T> + 'static) {
        let runtime = Self { future: RefCell::new(Box::pin(future)) };
        Self::poll(&Rc::new(runtime));
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
        assert_eq!(matches!(*future.state.borrow(), RuntimeState::Pending(None)), true);

        // wake future
        RuntimeFuture::wake(future.id, true);
        assert_eq!(matches!(*future.state.borrow(), RuntimeState::Competed(true)), true);

        // block on future
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        Runtime::block_on(async move { *has_run_clone.borrow_mut() = future.await; });
        assert_eq!(*has_run.borrow(), true);
    }

}
