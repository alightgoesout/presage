use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::AggregateEvent;

/// An aggregate represent a business entity that can evolve during the system execution.
///
/// It is identified by a unique _id_. An aggregate is always atomically modified to ensure
/// consistency.
///
/// # Associated types
///
/// * [Id](Self::Id) - the underlying type for the id that identifies a unique aggregate
/// * [CreationEvent](Self::CreationEvent) - the aggregate event when creating a new aggregate
/// * [UpdateEvent](Self::UpdateEvent) - the aggregate event when updating an existing aggregate
/// * [DeletionEvent](Self::DeletionEvent) - the aggregate event when deleting an existing aggregate
///
/// # Example
/// ```
/// # use presage::{Aggregate, AggregateEvent, Id};
/// #
/// # #[derive(AggregateEvent, serde::Serialize, serde::Deserialize)]
/// # #[presage(Todo)]
/// # pub struct TodoCreated { #[id] id: Id<Todo>, name: String }
/// # #[derive(AggregateEvent, serde::Serialize, serde::Deserialize)]
/// # #[presage(Todo)]
/// # pub enum TodoUpdated { Renamed { #[id] id: Id<Todo>, new_name: String }, Done(#[id] Id<Todo>) }
/// # #[derive(AggregateEvent, serde::Serialize, serde::Deserialize)]
/// # #[presage(Todo)]
/// # pub struct TodoDeleted(#[id] Id<Todo>);
/// #
/// pub struct Todo {
///     pub id: Id<Todo>,
///     pub name: String,
///     pub done: bool,
/// }
///
/// impl Aggregate for Todo {
///     type Id = u64;
///     type CreationEvent = TodoCreated;
///     type UpdateEvent = TodoUpdated;
///     type DeletionEvent = TodoDeleted;
///
///     fn id(&self) -> Id<Self> {
///         self.id
///     }
///
///     fn new(event: TodoCreated) -> Self {
///         Self {
///             id: event.id,
///             name: event.name,
///             done: false,
///         }
///     }
///
///     fn apply(&mut self, event: TodoUpdated) {
///         match event {
///             TodoUpdated::Renamed { new_name, .. } => self.name = new_name,
///             TodoUpdated::Done(_) => self.done = true,
///         }
///     }
/// }
/// ```
pub trait Aggregate: Sized + Send + Sync {
    /// The type of the aggregate id.
    type Id: Clone + Display + Send + Sync;

    /// The aggregate event that creates a new aggregate
    type CreationEvent: AggregateEvent<Aggregate = Self> + Send + Sync;

    /// The aggregate event that updates an existing aggregate
    type UpdateEvent: AggregateEvent<Aggregate = Self> + Send + Sync;

    /// The aggregate event that deletes an existing aggregate
    type DeletionEvent: AggregateEvent<Aggregate = Self> + Send + Sync;

    /// Getter for the aggregate id.
    fn id(&self) -> Id<Self>;

    /// Creates a new aggregate given the appropriate event.
    fn new(event: Self::CreationEvent) -> Self;

    /// Applies an update event to the aggregate.
    fn apply(&mut self, event: Self::UpdateEvent);
}

/// Wrapper type for the id of an aggregate.
///
/// This wrapper allows compile time checking of references between aggregates, and makes it easier
/// to change the wrapped type should the need arise.
///
/// It always implements the [Deref] and [Clone] traits, and optionally implements the [Copy],
/// [Debug], [Default], [Display], [Eq], [PartialEq], [Ord], [PartialOrd], [Hash], [Serialize],
/// [Deserialize] traits if they are implemented by the wrapped type.
#[repr(transparent)]
pub struct Id<A: Aggregate>(
    /// The wrapped id.
    pub A::Id,
);

impl<A: Aggregate> Deref for Id<A> {
    type Target = A::Id;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<A> Serialize for Id<A>
where
    A: Aggregate,
    A::Id: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'a, A> Deserialize<'a> for Id<A>
where
    A: Aggregate,
    A::Id: Deserialize<'a>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        A::Id::deserialize(deserializer).map(Id)
    }
}

impl<A> Default for Id<A>
where
    A: Aggregate,
    A::Id: Default,
{
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<A> Clone for Id<A>
where
    A: Aggregate,
{
    fn clone(&self) -> Self {
        Id(self.0.clone())
    }
}

impl<A> Copy for Id<A>
where
    A: Aggregate,
    A::Id: Copy,
{
}

impl<A> Debug for Id<A>
where
    A: Aggregate,
    A::Id: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Id({:?})", self.0)
    }
}

impl<A> Display for Id<A>
where
    A: Aggregate,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<A> PartialEq for Id<A>
where
    A: Aggregate,
    A::Id: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<A> Eq for Id<A>
where
    A: Aggregate,
    A::Id: Eq,
{
}

impl<A> PartialOrd for Id<A>
where
    A: Aggregate,
    A::Id: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<A> Ord for Id<A>
where
    A: Aggregate,
    A::Id: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl<A> Hash for Id<A>
where
    A: Aggregate,
    A::Id: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}
