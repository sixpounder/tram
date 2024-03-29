//! A simple event bus that can be cloned and shared across threads.
//!
//! ## Emit inside `on` blocks
//!
//! To re-emit events inside `on` closures do not clone the original bus, use the bus
//! reference provided to the closure instead.
//!
//! ## Example
//!
//! ```rust
//! use tram::{prelude::*, unsync::EventBus};
//! use std::{rc::Rc, cell::RefCell};
//!
//! #[derive(PartialEq, Eq, Hash)]
//! enum EventType {
//!     Start,
//!     Stop,
//! }
//!
//! #[derive(Debug, PartialEq)]
//! enum Status {
//!     Stopped,
//!     Started,
//! }
//!
//! let bus: EventBus<EventType, ()> = EventBus::unbound();
//!
//! let status = Rc::new(RefCell::new(Status::Stopped));
//! let status_closure = Rc::clone(&status);
//! let status_closure_2 = Rc::clone(&status);
//!
//! bus.on(EventType::Start, move |_bus, _| {
//!     *status_closure.borrow_mut() = Status::Started;
//! })
//! .expect("Failed to register listener");
//!
//! bus.on(EventType::Stop, move |_bus, _| {
//!     *status_closure_2.borrow_mut() = Status::Stopped;
//! })
//! .expect("Failed to register listener");
//!
//! bus.emit(EventType::Start).expect("Failed to emit");
//!
//! assert_eq!(*status.borrow(), Status::Started);
//! assert_eq!(bus.event_count(), 1);
//!
//! bus.emit(EventType::Stop).expect("Failed to emit");
//!
//! assert_eq!(*status.borrow(), Status::Stopped);
//! assert_eq!(bus.event_count(), 2);
//! ```
//!
//! ## Using it in threads
//!
//! ```
//! use tram::{prelude::*, sync::EventBus};
//! use std::sync::{Arc, Mutex};
//!
//! #[derive(PartialEq, Eq, Hash)]
//! enum EventType {
//!     Start,
//!     Stop,
//! }
//!
//! #[derive(Debug, PartialEq)]
//! enum Status {
//!     Stopped,
//!     Started,
//! }
//!
//! let bus: EventBus<EventType, ()> = EventBus::unbound();
//! let bus_clone = bus.clone();
//!
//! let status = Arc::new(Mutex::new(Status::Stopped));
//! let final_status = Arc::clone(&status);
//!
//! bus.on(EventType::Start, move |_bus, _| {
//!     let mut status_lock = status.lock().expect("Could not lock status");
//!     *status_lock = Status::Started;
//! }).expect("Could not register listener");
//!
//! let t2 = std::thread::spawn(move || {
//!     bus_clone.emit(EventType::Start).expect("Could not emit start event");
//! });
//!
//! t2.join().unwrap();
//!
//! let final_status_lock = final_status.lock().expect("Could not lock final status");
//! assert_eq!(*final_status_lock, Status::Started)
//! ```
//!
//! ## Passing data to events
//!
//! ```rust
//! use tram::{prelude::*, unsync::EventBus};
//! use std::{rc::Rc, cell::RefCell};
//!
//! #[derive(PartialEq, Eq, Hash)]
//! enum EventType {
//!     Start,
//!     Stop,
//! }
//!
//! #[derive(Debug, PartialEq)]
//! enum Status {
//!     Stopped,
//!     Started,
//! }
//!
//! let bus: EventBus<EventType, u8> = EventBus::unbound();
//! let status: Rc<RefCell<Option<u8>>> = Rc::new(RefCell::new(None));
//! let status_closure = Rc::clone(&status);
//!
//! bus.on(EventType::Start, move |_bus, startup_data| {
//!     *status_closure.borrow_mut() = Some(*startup_data.unwrap());
//! })
//! .unwrap();
//!
//! bus.emit_with_value(EventType::Start, Some(&123)).expect("Failed to emit");
//!
//! assert_eq!(*status.borrow(), Some(123));
//! assert_eq!(bus.event_count(), 1);
//! ```

pub mod prelude;
pub mod sync;
pub mod unsync;


