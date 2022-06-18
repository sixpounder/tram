use std::{cell::{RefCell, Cell}, collections::HashMap, hash::Hash};

#[derive(Debug, PartialEq)]
pub enum Error {
    /// Usually fired then a lock or a borrow cannot be obtained for a bus
    BusLock,

    /// Fired when a bus has reached its event count limit (if it has one)
    Disconnected,
}

pub trait EventEmitter<E, V> {
    /// Adds a listener `f` for and `event`
    fn on<F>(&self, event: E, f: F) -> Result<(), Error>
    where
        F: Fn(Option<&V>) + 'static;

    /// Emits an `event` with a `value` associated to it,
    /// firing all listeners connected to it via `on`.
    fn emit_with_value(&self, event: E, value: Option<&V>) -> Result<(), Error>;

    /// Emits an `event`, firing all listeners connected to it via `on`.
    ///
    /// When used this way the value passed to `on` closures will always be `None`.
    fn emit(&self, event: E) -> Result<(), Error> {
        self.emit_with_value(event, None)
    }
}

/// Inner implementation of a bus structure
pub struct BusRef<E, V> {
    marker: std::marker::PhantomData<E>,
    listeners: RefCell<std::collections::HashMap<E, Vec<Box<dyn Fn(Option<&V>)>>>>,
    emit_count: Cell<usize>,
    emit_limit: usize,
}

impl<E, V> BusRef<E, V> {
    pub(crate) fn unbound() -> Self {
        Self {
            marker: std::marker::PhantomData,
            listeners: RefCell::new(HashMap::new()),
            emit_count: Cell::new(0),
            emit_limit: 0,
        }
    }

    pub(crate) fn bound(max_emit_count: usize) -> Self {
        Self {
            marker: std::marker::PhantomData,
            listeners: RefCell::new(HashMap::new()),
            emit_count: Cell::new(0),
            emit_limit: max_emit_count,
        }
    }

    pub fn disconnected(&self) -> bool {
        let event_count = self.event_count();
        event_count != 0 && event_count == self.emit_limit
    }

    pub fn event_count(&self) -> usize {
        self.emit_count.get()
    }
}

impl<E, V> EventEmitter<E, V> for BusRef<E, V>
where
    E: Hash + Eq,
{
    /// Adds a listener `f` for and `event`
    fn on<F>(&self, event: E, f: F) -> Result<(), Error>
    where
        F: Fn(Option<&V>) + 'static,
    {
        let boxed_fn = Box::new(f);
        let mut listeners = self.listeners.borrow_mut();
        match listeners.get_mut(&event) {
            Some(existing_event) => {
                existing_event.push(boxed_fn);
            }
            None => {
                let v: Vec<Box<dyn Fn(Option<&V>) + 'static>> = vec![boxed_fn];
                listeners.insert(event, v);
            }
        }

        Ok(())
    }

    /// Emits an `event`, firing all listeners connected to it via `on`.
    ///
    /// When used this way the value passed to `on` closures will always be `None`.
    fn emit(&self, event: E) -> Result<(), Error> {
        self.emit_with_value(event, None)
    }

    /// Emits an `event` with a `value` associated to it,
    /// firing all listeners connected to it via `on`.
    fn emit_with_value(&self, event: E, value: Option<&V>) -> Result<(), Error> {
        if self.disconnected() {
            Err(Error::Disconnected)
        } else {
            let event_count = self.emit_count.get();
            self.emit_count.set(event_count + 1);
            let listeners = self.listeners.borrow();

            match listeners.get(&event) {
                Some(listeners_fns) => {
                    let _results = listeners_fns.iter().map(|l| l(value)).collect::<()>();
                    Ok(())
                }
                None => Ok(()),
            }
        }
    }
}


