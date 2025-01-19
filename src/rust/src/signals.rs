use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct Signal<T> {
    value: Rc<RefCell<T>>,
    // NOTE: since `FnMut` can mutate state it has to go behind a smart pointer
    subscribers: Rc<RefCell<Vec<Rc<RefCell<dyn FnMut() + 'static>>>>>,
}

impl<T: Clone + Send + 'static> Signal<T> {
    pub fn new(value: T) -> &'static Self {
        let signal = Self {
            value: Rc::new(RefCell::new(value)),
            subscribers: Default::default(),
        };
        &*Box::leak(Box::new(signal))
    }
    pub fn get(&self) -> T {
        self.value.borrow().clone()
    }
    pub fn set(&self, new_value: T) {
        // store value
        *self.value.borrow_mut() = new_value;

        // trigger effects
        self.subscribers.borrow_mut().iter().for_each(|f| {
            f.borrow_mut()();
        });
    }
    pub fn on(&self, mut cb: impl FnMut(T) + 'static) {
        // get callback
        let signal_clone = self.clone();
        let cb_ref = Rc::new(RefCell::new(move || {
            cb(signal_clone.get());
        }));

        // store callback
        self.subscribers.borrow_mut().push(cb_ref.to_owned());

        // trigger once
        cb_ref.borrow_mut()();
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_signals() {
        // create signal
        let logs: Rc<RefCell<Vec<u32>>> = Default::default();
        let signal = Signal::new(10);

        // create effects
        let logs_clone = logs.clone();
        signal.on(move |v| {
            logs_clone.borrow_mut().push(v);
        });
        let logs_clone = logs.clone();
        signal.on(move |v| {
            logs_clone.borrow_mut().push(v);
        });

        // update signal
        signal.set(20);
        signal.set(30);

        // check logs
        assert_eq!(*logs.borrow(), vec![10, 10, 20, 20, 30, 30]);
    }
}
