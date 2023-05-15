pub mod commands;
pub mod events;
pub mod views;

use presage::{Aggregate, Id};
use std::cmp::Ordering;
use time::OffsetDateTime;
use uuid::Uuid;

use events::{TodoCreated, TodoDeleted, TodoUpdated};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Todo {
    pub id: Id<Todo>,
    pub name: String,
    pub state: TodoState,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TodoState {
    New,
    Done {
        done_date: OffsetDateTime,
    },
    Archived {
        done_date: OffsetDateTime,
        archived_date: OffsetDateTime,
    },
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
            state: TodoState::New,
        }
    }

    fn apply(&mut self, event: TodoUpdated) {
        match event {
            TodoUpdated::Renamed { new_name, .. } => self.name = new_name,
            TodoUpdated::Done(_, done_date) => self.state = TodoState::Done { done_date },
            TodoUpdated::Archived {
                done_date,
                archived_date,
                ..
            } => {
                self.state = TodoState::Archived {
                    done_date,
                    archived_date,
                }
            }
        }
    }
}

impl PartialOrd for Todo {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Todo {
    fn cmp(&self, other: &Self) -> Ordering {
        self.state
            .cmp(&other.state)
            .then_with(|| self.name.cmp(&other.name))
            .then_with(|| self.id.cmp(&other.id))
    }
}
