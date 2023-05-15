use presage::{AggregateEvent, Id};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use super::Todo;

#[derive(Debug, Clone, Serialize, Deserialize, AggregateEvent)]
#[presage(Todo)]
pub struct TodoCreated {
    #[id]
    pub id: Id<Todo>,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, AggregateEvent)]
#[presage(aggregate = Todo, id = id)]
pub enum TodoUpdated {
    Renamed {
        id: Id<Todo>,
        old_name: String,
        new_name: String,
    },
    Done(#[id] Id<Todo>, OffsetDateTime),
    Archived {
        id: Id<Todo>,
        done_date: OffsetDateTime,
        archived_date: OffsetDateTime,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, AggregateEvent)]
#[presage(aggregate = Todo)]
pub struct TodoDeleted(#[id] pub Id<Todo>);
