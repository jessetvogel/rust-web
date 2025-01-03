

### Runtime flow

```js
// 1. Register a callback and invoke `fetch` that triggers the callback when is finishes
[Log] create_async_callback future_id=0 -> [Log] create_callback id=0 && [Log] js_invoke `fetch` id=0

// 2. Use the `block_on` method that calls `poll` on the future that in turn sets `FutureState` to `Pending(waker)`
[Log] runtime block on -> [Log] future poll -> [Log] poll future pending

// 3. When the `fetch` callback is triggered, schedule a `setTimeout(0)` callback that calls future poll
[Log] handle_callback id=0 -> [Log] waker wake -> [Log] create_callback id=2  && [Log] js_invoke `setTimeout(0)` id=0

// 4. When the `setTimeout(0)` callback is triggered, resolve future to `Poll::Ready(T)`
[Log] handle_callback id=0 -> [Log] future poll -> [Log] poll future completed
```


### Implementation quirks

1. Updating `FutureState` in 2 different places
  - It's updated in the `FutureTask::wake` function
  - It's also updated in the `Future` trait impl

2. Using the `thread_local` macro to access the future
  - The `thread_local` macro is used for shared ownership of the future
  - It stores the `FutureState` that contains the `waker` object
  - The `FutureTask` is not moved and the `.await` syntax still works

3. (Maybe necessary) Calling `poll` in `wake_fn` through Javascript
  - The `wake_fun` function creates a callback that does `Runtime::poll(&future)`
  - It schedules it immediately with `setTimeout(0)` instead of directly calling
