

### Runtime flow

```js
// 1. Register a callback and invoke `fetch` that triggers the callback when is finishes
[Log] create_async_callback future_id=0 -> [Log] create_callback id=0 && [Log] js_invoke `fetch` id=0

// 2. Use the `block_on` method and `await` the future inside
// When the future is awaited it calls `poll` function that sets `FutureState` to `Pending(waker)`
[Log] runtime block on -> [Log] future poll -> [Log] poll future pending

// 3. When the `fetch` callback is triggered, schedule a `setTimeout(0)` callback that calls future poll
[Log] handle_callback id=0 -> [Log] waker wake -> [Log] create_callback id=2  && [Log] js_invoke `setTimeout(0)` id=0

// 4. When the `setTimeout(0)` callback is triggered, resolve future to `Poll::Ready(T)`
[Log] handle_callback id=0 -> [Log] future poll -> [Log] poll future completed
```


### Implementation quirks

1. Updating `FutureState` in 2 different places
  - Notes: It's updated in `create_async_callback` and in the `Future` trait impl
  - Explanation: `create_async_callback` has the `result` value and `Future` has access to the concrete `self` type

2. Calling `poll` in `wake_fn` through a Javascript callback instead of directly calling
  - Notes: The `wake_fn` function schedules a callback with `setTimeout(0)` that does `Runtime::poll(&future)`
  - Explanation: `Runtime::poll` has a mutable borrow that still holds when `wake_fn` tries to borrow again
