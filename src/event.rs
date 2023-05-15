use async_trait::async_trait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;

use crate::{Aggregate, Commands, Error, Id};

/// An event represent something that happened in the past.
///
/// It is serialized using the `serde` crate when being dispatched, so it can be shared between
/// independent parts of the system. An event must thus implement [Serialize] and
/// [Deserialize](serde::Deserialize).
///
/// # Associated constant
///
/// * [NAME](Self::NAME) - the unique name of the event
///
/// # Example
///
/// ```
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct SystemStarted(std::time::Instant);
///
/// impl presage::Event for SystemStarted {
///     const NAME: &'static str = "system-started";
/// }
/// ```
pub trait Event: Serialize + DeserializeOwned {
    /// The name of the event. Must be unique.
    const NAME: &'static str;

    /// Serializes and event into a [SerializedEvent].
    fn serialize(self) -> Result<SerializedEvent, Error> {
        Ok(SerializedEvent {
            name: Self::NAME,
            value: serde_json::to_value(self)?,
        })
    }
}

/// An [Event] that creates, updates, or deletes an aggregate.
///
/// # Associated type
///
/// * [Aggregate](Self::Aggregate) - the type of the affected aggregate
///
/// # Example
///
/// ```
/// use presage::{AggregateEvent, Event, Id};
///
/// #[derive(serde::Serialize, serde::Deserialize)]
/// pub struct TodoCreated {
///     pub id: Id<Todo>,
///     pub name: String,
/// }
///
/// impl Event for TodoCreated {
///     const NAME: &'static str = "todo-created";
/// }
///
/// impl AggregateEvent for TodoCreated {
///     type Aggregate = Todo;
///
///     fn id(&self) -> Id<Self::Aggregate> {
///         self.id
///     }
/// }
/// ```
pub trait AggregateEvent: Event {
    /// The [Aggregate] associated with the event.
    type Aggregate: Aggregate;

    /// The id of the affected aggregate.
    fn id(&self) -> Id<Self::Aggregate>;
}

/// An event that has been serialized to be issued by a command.
///
/// Can be created from an [Event].
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SerializedEvent {
    name: &'static str,
    value: Value,
}

impl SerializedEvent {
    /// Tries to deserialize to a concrete [Event].
    pub fn deserialize<E: Event>(self) -> Result<E, Error> {
        Ok(serde_json::from_value(self.value)?)
    }

    /// The name of the serialized event
    pub fn name(&self) -> &'static str {
        self.name
    }
}

/// Wrapper for a [Vec] of [serialized events](SerializedEvent).
#[derive(Debug, Clone, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct Events(pub Vec<SerializedEvent>);

impl Events {
    /// Creates a new empty [Events].
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Serializes an event and adds it to the wrapped [Vec].
    pub fn add(&mut self, event: impl Event) -> Result<(), Error> {
        self.0.push(event.serialize()?);
        Ok(())
    }
}

impl FromIterator<SerializedEvent> for Events {
    fn from_iter<T: IntoIterator<Item = SerializedEvent>>(iter: T) -> Self {
        Events(iter.into_iter().collect())
    }
}

impl IntoIterator for Events {
    type Item = SerializedEvent;
    type IntoIter = <Vec<SerializedEvent> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Creates a new [Events] containing the events passed as arguments.
#[macro_export]
macro_rules! events {
    ($($events: expr),* $(,)?) => {
        $crate::Events(vec![$($crate::Event::serialize($events)?),*])
    }
}

/// Reacts to issued [events](Event).
///
/// Can issue new [commands](crate::Command).
///
/// # Type arguments
///
/// * `C` - the context for this handler
/// * `E` - the type of errors returned if the handler fails
#[async_trait]
pub trait EventHandler<C, E>: Send + Sync {
    /// The names of the handled events.
    fn event_names(&self) -> &[&'static str];

    /// Handles an event with the given context.
    async fn handle(&self, context: &mut C, event: &SerializedEvent) -> Result<Commands, E>;
}
