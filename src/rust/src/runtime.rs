use std::{
    any::Any,
    cell::RefCell,
    collections::HashMap,
    future::Future,
    mem::ManuallyDrop,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker}
};

use crate::callbacks::create_callback;
use crate::invoke::Js;

thread_local! {
    static STATE_MAP: RefCell<HashMap<u32, Box<dyn Any>>> = Default::default(); // Cast: Rc<RefCell<RuntimeState<T>>>
}

pub struct RuntimeState<T> { completed: bool, waker: Option<Waker>, result: Option<T>, }
pub struct RuntimeFuture<T> { id: u32, state: Rc<RefCell<RuntimeState<T>>>, }
pub struct Runtime<T> { future: RefCell<Pin<Box<dyn Future<Output = T>>>>, }

impl<T> Future for RuntimeFuture<T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut future = self.state.borrow_mut();
        let poll = if future.completed && future.result.is_some() {
            Poll::Ready(future.result.take().unwrap())
        } else {
            future.waker = Some(cx.waker().to_owned());
            Poll::Pending
        };
        return poll;
    }
}

// https://rust-lang.github.io/async-book/02_execution/03_wakeups.html
impl <T: 'static> RuntimeFuture<T> {
    pub fn new() -> Self {

        // using `Number.MAX_SAFE_INTEGER` exceeds u32
        let future_id = Js::invoke("return Math.random() * (2 ** 32)", &[]).to_num().unwrap();
        let state = RuntimeState { completed: false, waker: None, result: None, };
        let state_arc = Rc::new(RefCell::new(state));
        STATE_MAP.with_borrow_mut(|s| {
            s.insert(future_id as u32, Box::new(state_arc.clone()));
        });

        Self { id: future_id as u32, state: state_arc }
    }

    pub fn id(&self) -> u32 { self.id }

    pub fn wake(future_id: u32, result: T) {
        STATE_MAP.with_borrow_mut(|s| {
            let future_any = s.get_mut(&future_id).unwrap();
            let future = future_any.downcast_mut::<Rc<RefCell<RuntimeState<T>>>>().unwrap();
            let mut p = future.borrow_mut();
            if let Some(waker) = p.waker.take() { waker.wake(); }
            p.completed = true;
            p.result = Some(result);
            drop(p);
            s.remove(&future_id).unwrap();
        });
    }
}

impl<T: 'static> Runtime<T> {

    fn poll(task: &Rc<Self>) {
        let waker = Self::waker(&task);
        let waker_forget = ManuallyDrop::new(waker);
        let context = &mut Context::from_waker(&waker_forget);
        let _poll = task.future.borrow_mut().as_mut().poll(context);
    }

    fn waker(task: &Rc<Self>) -> Waker {

        fn clone_fn<T: 'static>(ptr: *const ()) -> RawWaker {
            let _task = unsafe { Rc::<Runtime<T>>::from_raw(ptr as *const _) };
            let _ = ManuallyDrop::new(_task).clone();

            RawWaker::new(ptr, waker_vtable::<T>())
        }
        fn wake_fn<T: 'static>(ptr: *const ()) {
            let _task = unsafe { Rc::<Runtime<T>>::from_raw(ptr as *const _) };
            let function_ref = create_callback(move |_| { Runtime::poll(&_task); });
            Js::invoke("window.setTimeout({},0)", &[function_ref.into()]);
        }
        fn drop_fn<T>(ptr: *const ()) {
            let _task = unsafe { Rc::<Runtime<T>>::from_raw(ptr as *const _) };
            drop(_task);
        }
        fn waker_vtable<T: 'static>() -> &'static RawWakerVTable {
            &RawWakerVTable::new(clone_fn::<T>, wake_fn::<T>, wake_fn::<T>, drop_fn::<T>)
        }
        let ptr = &**task as *const _;
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
        assert_eq!(future.state.borrow().result, None);
        assert_eq!(future.state.borrow().completed, false);
        assert_eq!(future.state.borrow().waker.is_some(), false);

        // wake future
        RuntimeFuture::wake(future.id, true);
        assert_eq!(future.state.borrow().result, Some(true));
        assert_eq!(future.state.borrow().completed, true);
        assert_eq!(future.state.borrow().waker.is_some(), false);

        // block on future
        let has_run = Rc::new(RefCell::new(false));
        let has_run_clone = has_run.clone();
        Runtime::block_on(async move { *has_run_clone.borrow_mut() = future.await; });
        assert_eq!(*has_run.borrow(), true);
    }

}
