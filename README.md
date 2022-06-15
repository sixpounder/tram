A simple event bus that can be cloned and shared across threads

# Example

```rust
[derive(PartialEq, Eq, Hash)]
enum EventType {
    Start,
    Stop,
}

#[derive(Debug, PartialEq)]
enum Status {
    Stopped,
    Started,
}

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
```

## Using it in threads

```rust
[derive(PartialEq, Eq, Hash)]
enum EventType {
    Start,
    Stop,
}

#[derive(Debug, PartialEq)]
enum Status {
    Stopped,
    Started,
}



```
