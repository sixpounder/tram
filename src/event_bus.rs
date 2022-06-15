use std::{
    cell::Cell,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
};

#[derive(Debug, PartialEq)]
pub enum Error {
    BusLock,
}

pub struct BusRef<E> {
    marker: std::marker::PhantomData<E>,
    listeners: std::collections::HashMap<E, Vec<Box<dyn Fn()>>>,
    event_count: Cell<usize>,
}

impl<E> BusRef<E> {
    fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
            listeners: HashMap::new(),
            event_count: Cell::new(0),
        }
    }
}

/// An event bus that can be cloned and shared across threads
///
/// # Example
///
/// ```ignore
/// # use crate::EventBus;
/// # use std::{cell::RefCell, rc::Rc};
/// #[derive(PartialEq, Eq, Hash)]
/// enum EventType {
///     Start,
///     Stop
/// }
///
/// #[derive(Debug, PartialEq)]
/// enum Status {
///     Stopped,
///     Started
/// }
///
/// let bus: EventBus<EventType> = EventBus::new();
/// let status = Rc::new(RefCell::new(Status::Stopped));
/// let status_closure = Rc::clone(&status);
/// bus.on(EventType::Start, move || {
///     *status_closure.borrow_mut() = Status::Started;
/// })
/// .expect("Failed to listen for this event");
///
/// bus.emit(EventType::Start).expect("Failed to emit");
///
/// assert_eq!(*status.borrow(), Status::Started);
/// assert_eq!(bus.event_count(), 1);
/// ```
pub struct EventBus<E> {
    bus: Arc<Mutex<BusRef<E>>>,
}

impl<E> EventBus<E> {
    pub fn new() -> Self {
        Self {
            bus: Arc::new(Mutex::new(BusRef::new())),
        }
    }
}

impl<E> EventBus<E>
where
    E: Eq + Hash,
{
    /// Adds a listener `f` for and `event`
    pub fn on<F: Fn() + 'static>(&self, event: E, f: F) -> Result<(), Error> {
        if let Ok(mut bus_lock) = self.bus.lock() {
            let boxed_fn = Box::new(f);

            match bus_lock.listeners.get_mut(&event) {
                Some(existing_event) => {
                    existing_event.push(boxed_fn);
                }
                None => {
                    let v: Vec<Box<dyn Fn() + 'static>> = vec![boxed_fn];
                    bus_lock.listeners.insert(event, v);
                }
            }

            Ok(())
        } else {
            Err(Error::BusLock)
        }
    }

    /// Emits an `event`, firing all listeners connected to it via `on`
    pub fn emit(&self, event: E) -> Result<(), Error> {
        if let Ok(bus_lock) = self.bus.lock() {
            let event_count = bus_lock.event_count.get();
            bus_lock.event_count.set(event_count + 1);

            match bus_lock.listeners.get(&event) {
                Some(listeners) => {
                    let _results = listeners.iter().map(|l| l()).collect::<()>();
                    Ok(())
                }
                None => Ok(()),
            }
        } else {
            Err(Error::BusLock)
        }
    }

    pub fn event_count(&self) -> usize {
        self.bus.lock().unwrap().event_count.get()
    }
}

impl<E> Clone for EventBus<E> {
    fn clone(&self) -> Self {
        Self {
            bus: Arc::clone(&self.bus),
        }
    }
}

unsafe impl<E> Send for EventBus<E> where E: Send {}

unsafe impl<E> Sync for EventBus<E> where E: Sync {}

#[cfg(test)]
mod test {
    use super::*;

    use std::cell::RefCell;
    use std::rc::Rc;
    
    #[derive(PartialEq, Eq, Hash)]
    enum EventType {
        Start,
        Stop,
    }

    #[derive(Debug, PartialEq)]
    enum Status {
        Stopped,
        Started,
    }

    #[test]
    fn create_bus() {
        let _bus: EventBus<u8> = EventBus::new();
    }

    #[test]
    fn listen_emit_api() {
        let bus: EventBus<EventType> = EventBus::new();
        let status = Rc::new(RefCell::new(Status::Stopped));
        let status_closure = Rc::clone(&status);
        let status_closure_2 = Rc::clone(&status);
        bus.on(EventType::Start, move || {
            *status_closure.borrow_mut() = Status::Started;
        })
        .unwrap();

        bus.on(EventType::Stop, move || {
            *status_closure_2.borrow_mut() = Status::Stopped;
        })
        .unwrap();

        bus.emit(EventType::Start).expect("Failed to emit");

        assert_eq!(*status.borrow(), Status::Started);
        assert_eq!(bus.event_count(), 1);

        bus.emit(EventType::Stop).expect("Failed to emit");

        assert_eq!(*status.borrow(), Status::Stopped);
        assert_eq!(bus.event_count(), 2);
    }

    #[test]
    fn listen_emit_api_repeat() {
        let bus: EventBus<u8> = EventBus::new();
        let status = Rc::new(RefCell::new(0));
        let status2 = Rc::clone(&status);
        bus.on(1u8, move || {
            *status2.borrow_mut() += 1;
        })
        .unwrap();

        bus.emit(1).expect("Failed to emit");
        bus.emit(1).expect("Failed to emit");
        bus.emit(1).expect("Failed to emit");
        bus.emit(1).expect("Failed to emit");
        bus.emit(200).expect("Failed to emit");

        assert_eq!(*status.borrow(), 4);
        assert_eq!(bus.event_count(), 5);
    }
    
    #[test]
    fn threaded() {
        let bus: EventBus<EventType> = EventBus::new();
        let bus_clone = bus.clone();
        let status = Arc::new(Mutex::new(Status::Stopped));
        let final_status = Arc::clone(&status);
        
        let t1 = std::thread::spawn(move || {
            bus.on(EventType::Start, move || {
                let mut status_lock = status.lock().unwrap();
                *status_lock = Status::Started;
            }).unwrap();
        });

        let t2 = std::thread::spawn(move || {
            bus_clone.emit(EventType::Start).unwrap();
        });
        
        t1.join().unwrap();
        t2.join().unwrap();
        
        let final_status_lock = final_status.lock().unwrap();
        assert_eq!(*final_status_lock, Status::Started)
    }
}


