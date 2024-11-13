
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Signal<T> {
    value: Arc<Mutex<T>>,
    // NOTE: since `FnMut` can mutate state it has to go behind a smart pointer
    subscribers: Arc<Mutex<Vec<Arc<Mutex<dyn FnMut() + Send + 'static>>>>>
}

impl<T: Clone + Send + 'static> Signal<T> {
    pub fn new(value: T) -> Self {
        Self { value: Arc::new(Mutex::new(value)), subscribers: Default::default(), }
    }
    pub fn get(&self) -> T {
        self.value.lock().map(|s| s.to_owned()).unwrap()
    }
    pub fn set(&self, new_value: T) {
        // store value
        self.value.lock().map(|mut s| {
            *s = new_value;
        }).unwrap();

        // trigger effects
        self.subscribers.lock().map(|s| {
            s.iter().for_each(|e| { e.lock().map(|mut f| { f(); }).unwrap(); })
        }).unwrap();
    }
    pub fn on(&self, mut cb: impl FnMut(T) + Send + 'static) {

        // get callback
        let signal_clone = self.clone();
        let cb_ref = Arc::new(Mutex::new(move || { cb(signal_clone.get()); }));

        // store callback
        self.subscribers.lock().map(|mut c| { c.push(cb_ref.to_owned()); }).unwrap();

        // trigger once
        cb_ref.lock().map(|mut f| { f(); }).unwrap();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_signals() {

        // create signal
        let logs: Arc<Mutex<Vec<u32>>> = Default::default();
        let signal = Signal::new(10);

        // create effects
        let logs_clone = logs.clone();
        signal.on(move |v| { logs_clone.lock().map(|mut s| { s.push(v); }).unwrap(); });
        let logs_clone = logs.clone();
        signal.on(move |v| { logs_clone.lock().map(|mut s| { s.push(v); }).unwrap(); });

        // update signal
        signal.set(20);
        signal.set(30);

        // check logs
        let received = logs.lock().map(|s| s.to_owned()).unwrap();
        assert_eq!(received, vec![10, 10, 20, 20, 30, 30]);
    }

}
