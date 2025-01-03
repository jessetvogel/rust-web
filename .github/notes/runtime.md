

### Runtime flow
```js
// 1. register a callback and invoke `fetch` that triggers the callback when is done
[Log] create_async_callback future_id=0 -> [Log] create_callback id=0 && [Log] js_invoke `fetch` id=0

// 2. the `block_on` method calls the `poll` on the future that set `FutureState` to `Pending(waker)`
[Log] runtime block on -> [Log] future poll -> [Log] poll future pending

// 3. when the `fetch` callback is done, schedule a `setTimeout(0)` callback that calls future poll
[Log] handle_callback id=0 -> [Log] waker wake -> [Log] create_callback id=2  && [Log] js_invoke `setTimeout(0)` id=0

// 4. when the `setTimeout(0)` callback is done, resolve future to `Poll::Ready(T)`
[Log] handle_callback id=0 -> [Log] future poll -> [Log] poll future completed
```

### Notes about `thread_local`

- The `thread_local` macro has `FutureState` that has the `waker` object (used to trigger the future's `poll` method)
- Since `FutureTask` is moved to `thread_local`, it's still owned and the `.await` syntax still works

### Runtime quirks
1. Updating the `FutureState` in both `FutureTask::wake` and in the `Future` trait impl
2. Using `thread_local` instead of accessing the future directly
3. (Maybe necessary) Using `setTimeout(0)` instead of directly calling `Runtime::poll(&future)`
