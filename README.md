A simple event bus that can be cloned and shared across threads.

## Limitations

At present the only limitation is the inability to use the same `EventBus` inside
`on` blocks because it would lead to a deadlock for `sync` variants and to an `Error`
for `unsync` variant (thou this is a work in progress).

## Example

```rust
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

let mut bus: EventBus<EventType> = EventBus::unbound();

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
```

## Using it in threads

```rust
let mut bus: EventBus<EventType, ()> = EventBus::unbound();
let bus_clone = bus.clone();

let status = Arc::new(Mutex::new(Status::Stopped));
let final_status = Arc::clone(&status);

let t1 = std::thread::spawn(move || {
    bus.on(EventType::Start, move |_| {
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
```

## Passing data to events

```rust
let bus: EventBus<EventType, u8> = EventBus::unbound();
let status: Rc<RefCell<Option<u8>>> = Rc::new(RefCell::new(None));
let status_closure = Rc::clone(&status);

bus.on(EventType::Start, move |startup_data| {
    *status_closure.borrow_mut() = Some(*startup_data.unwrap());
})
.unwrap();

bus.emit_with_value(EventType::Start, Some(&123)).expect("Failed to emit");

assert_eq!(*status.borrow(), Some(123));
assert_eq!(bus.event_count(), 1);
```
