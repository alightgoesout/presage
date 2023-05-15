use presage::{async_trait, Aggregate, AggregateEvent, Event, EventWriter, Id, SerializedEvent};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

use crate::todo::events::{TodoCreated, TodoDeleted, TodoUpdated};
use crate::todo::views::TodosSummary;
use crate::todo::{Todo, TodoState};
use crate::Error;

#[derive(Default)]
pub struct TodoContext {
    todos: HashMap<Id<Todo>, Todo>,
    summary: TodosSummary,
}

impl TodoContext {
    pub fn get(&self, id: Id<Todo>) -> Option<Todo> {
        self.todos.get(&id).cloned()
    }

    pub fn add(&mut self, todo: Todo) -> Result<(), Error> {
        match self.todos.entry(todo.id) {
            Entry::Vacant(entry) => {
                entry.insert(todo);
                Ok(())
            }
            _ => Err(Error(format!("A todo with id {} already exists", todo.id))),
        }
    }

    pub fn update(&mut self, todo: Todo) -> Result<(), Error> {
        match self.todos.entry(todo.id) {
            Entry::Occupied(mut entry) => {
                entry.insert(todo);
                Ok(())
            }
            _ => Err(Error(format!("Todo {} was not found", todo.id))),
        }
    }

    pub fn delete(&mut self, id: Id<Todo>) {
        self.todos.remove(&id);
    }

    pub fn list_visible_todos(&self) -> Vec<Todo> {
        let mut todos: Vec<_> = self
            .todos
            .values()
            .filter(|todo| !matches!(todo.state, TodoState::Archived { .. }))
            .cloned()
            .collect();
        todos.sort();
        todos
    }

    pub fn list_archived_todos(&self) -> Vec<Todo> {
        let mut todos: Vec<_> = self
            .todos
            .values()
            .filter(|todo| matches!(todo.state, TodoState::Archived { .. }))
            .cloned()
            .collect();
        todos.sort();
        todos
    }

    pub fn summary(&self) -> TodosSummary {
        self.summary
    }

    pub fn save_summary(&mut self, summary: TodosSummary) {
        self.summary = summary;
    }
}

pub struct TodoEventWriter;

#[async_trait]
impl EventWriter<TodoContext, Error> for TodoEventWriter {
    fn event_names(&self) -> &[&'static str] {
        &[TodoCreated::NAME, TodoUpdated::NAME, TodoDeleted::NAME]
    }

    async fn write(&self, context: &mut TodoContext, event: &SerializedEvent) -> Result<(), Error> {
        if event.name() == TodoCreated::NAME {
            let todo = Todo::new(event.clone().deserialize()?);
            context.add(todo)?;
        } else if event.name() == TodoUpdated::NAME {
            let event: TodoUpdated = event.clone().deserialize()?;
            if let Some(mut todo) = context.get(event.id()) {
                todo.apply(event);
                context.update(todo)?;
            }
        } else {
            let event: TodoDeleted = event.clone().deserialize()?;
            context.delete(event.id());
        }
        Ok(())
    }
}
