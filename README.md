# Présage \pʁe.zaʒ\

_A lightweight Rust library for designing event-based systems_

***

## Concepts

In an event-based systems, anything that can happen is modeled with an event. Business entities are
designed with aggregates, that are atomically modified with events.

Présage is freely inspired by concepts like domain-driven design or command-query responsibility
segregation, but tries to be agnostic by making as few assumptions as possible. Specifically, it
does not tie you to any persistence approach.

### Aggregates

An aggregate represent a business entity that can evolve during the system execution. It is
identified by a unique _id_. An aggregate is always atomically modified to ensure consistency.

To define an aggregate, you need to create a structure or an enumeration that will serve as an
aggregate root and that implements the `Aggregate` trait:

```rust
use presage::{Aggregate, Id};
use uuid::Uuid;

pub struct Todo {
    pub id: Id<Todo>,
    pub name: String,
    pub done: bool,
}

impl Aggregate for Todo {
    type Id = Uuid;
    type CreationEvent = TodoCreated;
    type UpdateEvent = TodoUpdated;
    type DeletionEvent = TodoDeleted;

    fn id(&self) -> Id<Self> {
        self.id
    }

    fn new(event: TodoCreated) -> Self {
        Self {
            id: event.id,
            name: event.name,
            done: false,
        }
    }

    fn apply(&mut self, event: TodoUpdated) {
        match event {
            TodoUpdated::Renamed { new_name, .. } => self.name = new_name,
            TodoUpdated::Done(_) => self.done = true,
        }
    }
}
```

You must specify the type of the id. It can be anything, as long as each
aggregate has a unique value. When referencing an aggregate, `Id<A>` should be used as it allows
compile-tile checks. If the type of the id has the `Copy` trait, `Id<A>` will also have it. You need
to implement the `id` function that returns the id of an aggregate.

You also need to specify three aggregate event types: one for creating, one for updating, and one
for deleting the aggregate (see [events](#events) for how to define events), as well as a function
to create a new aggregate (`new`) and a function to update an existing aggregate (`apply`).

### Events

An event represent something that happened in the past . It is serialized using the `serde` crate
when being dispatched, so it can be shared between independent parts of the system. An event must
thus implement `serde::Serialize` and `serde::Deserialize`.

Simple events are defined by implementing the `Event` trait on a structure or an enumeration:

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct SystemStarted(Instant);

impl presage::Event for SystemStarted {
    const NAME: &'static str = "system-started";
}
```

The `NAME` constant identifies the type of the event and must be unique.

To modify the state of the system, specialized events are needed: aggregate events. An aggregate
event affects a single aggregate that must be referenced by its id. If multiple aggregates are
modified, multiple aggregate events must be issued. The `AggregateEvent` trait allows the
specification of aggregate events:

```rust
use presage::{AggregateEvent, Event, Id};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TodoCreated {
    pub id: Id<Todo>,
    pub name: String,
}

impl Event for TodoCreated {
    const NAME: &'static str = "todo-created";
}

impl AggregateEvent for TodoCreated {
    type Aggregate = Todo;

    fn id(&self) -> Id<Self::Aggregate> {
        self.id
    }
}
```

### Commands

Commands are requests to modify the system. They are implemented as structures or enumerations that
contain the required information. It is associated with a [command handler](#command-handlers)
which produces events when executed.

```rust
#[derive(Debug)]
pub struct CreateTodo {
    pub id: Id<Todo>,
    pub name: String,
}

impl presage::Command for CreateTodo {
    const NAME: &'static str = "create-todo";
}
```

## Handlers

Once you have defined commands and events, you need to write _handlers_ for the system to actually
do something. A command handler takes a command and returns events, while an event handler takes an
event and returns commands.

### Context

All handlers require a mutably borrowed context to be passed with the command or event it is
handling. The context can be anything you need, and its role is to manage inputs and outputs, for
instance to load an aggregate in a command handler.

When defining a handler, you can specify the exact type of the context used a runtime, or use
generics to specify the traits the handler is required to implement.

### Command handlers

A command handler handles a single command type, identified by its name. It implements
the `CommandHandler` trait:

```rust
use presage::{CommandHandler, Error, Events, BoxedCommand};

struct CreateTodoHandler;

#[async_trait::async_trait]
impl<C, E> CommandHandler<C, E> for CreateTodoHandler
    where
        E: From<Error>,
{
    fn command_name(&self) -> &'static str {
        "create-todo"
    }

    async fn handle(&self, _context: &mut C, command: BoxedCommand) -> Result<Events, E> {
        let CreateTodo { id, name } = command.downcast()?;
        Ok(events!(TodoCreated { id, name }))
    }
}
```

### Event handlers

An event handler can be associated with multiple event types, and implements the `EventHandler`
trait:

```rust
use presage::{Commands, Error, EventHandler, SerializedEvent};
use crate::{LoadTodoView, SaveTodoView, TodoView, TodoCreated, TodoUpdated};

struct CreateTodoViewOnTodoEvent;

#[async_trait::async_trait]
impl EventHandler<C, E> for CreateTodoViewOnTodoEvent
    where
        C: LoadTodoView + SaveTodoView,
        E: From<Error>,
{
    fn event_names(&self) -> &[&'static str] {
        &[TodoCreated::NAME, TodoUpdated::NAME]
    }

    async fn handle(&self, context: &mut C, event: &SerializedEvent) -> Result<Commands, E> {
        if event.name == TodoCreated::NAME {
            let TodoCreated { id, name } = event.deserialize()?;
            context.save(TodoView { id, name, done: false }).await?;
        } else {
            let todo_updated: TodoUpdated = event.deserialize()?;
            // …
        }
        Ok(Default::default())
    }
}
```

## Macros

To simplify writing commands, events, and handlers, a few derive and attribute macros are available
with the `derive` feature (active by default).

### Deriving traits

The `Command`, `Event`, and `AggregateEvent` can all be derived. By default, the name of the command
or event is derived from the name of the type by converting it to kebab case (e.g., `TodoCreated`
becomes `todo-created`). If you want to specify another name, you can use
the `#[presage(name = "name")]` attribute on the type for which the trait is derived.

To properly derive the `AggregateEvent` trait, more information is required. The type of the
associated aggregate must be specified by using the `#[presage]` attribute:`#[presage(Todo)]`
or `#[presage(aggregate = Todo)]`. The field containing must be specified, either by declaring it in
on the type with `#[presage(id = id_field)]` or by adding the `#[id]` attribute on the field. If the
event is an enumeration, a mix of both approach can be used:

```rust
#[derive(Serialize, Deserialize, AggergateEvent)]
#[presage(aggregate = SomeAggregate, id = id)]
enum SomeAggregateEvent {
    // No field with the #[id] attribute, the field specified on the type is used
    Variant1 { id: Id<SomeAggregate>, /* … */ },
    // The id field has a different name, we need to add the #[id] attribute
    Variant2 { #[id] id_field: Id<SomeAggregate>, /* … */ },
    // Fields of a tuple variant have no name, we need to add the #[id] attribute
    Variant3(#[id] Id<SomeAggregate>, /* … */),
}
```

### Handlers

Both `CommandHandler` and `EventHandler` can be automatically created using the `#[command_handler]`
and `#[event_handler]` attributes on a function, respectively.

For a command handler, the function must have two parameters: the context and the handled command.
For instance, the `CreateTodoHandler` defined above can be written like this:

```rust
#[command_handler]
pub async fn create_todo<C, E>(_context: &mut C, CreateTodo { id, name }: CreateTodo) -> Result<Events, E>
    where
        E: From<presage::Error>,
{
    Ok(events!(TodoCreated { id, name }))
}
```

The `#[event_handler]` can be used with two different function signatures. If it only handles one
type of event, the function can use the actual event type as second parameter, otherwise it needs to
specify the event names in the attribute and use `&SerializedEvent` for the type of the second
parameter:

```rust
#[event_handler]
pub async fn simple_event_handler<C, E>(context: &mut C, event: SomeEvent) -> Result<Commands, E>
    where
        E: From<presage::Error>,
{
    // …
}

#[event_handler(events = [SomeEvent, "event-name"])]
pub async fn multiple_events_handler<C, E>(context: &mut C, event: &SerializedEvent) -> Result<Commands, E>
    where
        E: From<presage::Error>,
{
    // …
}
```

For both type of handlers, the macro needs to extract the error type from the function signature. If
you are using a type alias, you can specify the error type in the attribute:

```rust
#[command_handler(error = MyError)]
pub async fn some_handler(context: &mut MyContext, command: SomeCommand) -> MyResult<Events> {
    // …
}
```

## Command bus

The command bus is the main entry point when running a system using présage. It takes a context and
a command that is then executed. The resulting events are persisted, then any matching event handler
is executed. Those event handlers can return new commands which are also executed. The process
continues as long as new commands are issued. Be careful to avoid an infinite loop!

### Configuration

A command bus can be created using the `new` function (or the `Default` implementation). It can then
be configured by providing a `Configuration`.

```rust
let command_bus = CommandBus::new()
    .configure(
        Configuration::new()
            .event_writer( & some_writer)
            .event_handler( & some_event_handler)
            .command_handler( & some_command_handler)
    );
```

## Persistence

The modifications of the system must all be modeled using events. These modifications are persisted
using the trait `EventWriter`. The actual implementation can persist the results of applying the
events, or it can persist the events themselves. With présage you can choose the approach you
prefer, and you can even mix the two approaches.

## Examples

The [examples](examples) folder contains a simple command line todo application using présage. It
illustrates most of the concepts discussed in this file.

## License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
