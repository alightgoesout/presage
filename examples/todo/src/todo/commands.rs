use presage::{command_handler, events, Command, Event, Events, Id};
use time::OffsetDateTime;

use super::events::{TodoCreated, TodoUpdated};
use super::{Todo, TodoState};
use crate::persistence::TodoContext;
use crate::todo::events::TodoDeleted;
use crate::Error;

#[derive(Debug, Command)]
pub struct CreateTodo {
    pub id: Id<Todo>,
    pub name: String,
}

#[command_handler]
pub async fn create_todo(
    _context: &mut TodoContext,
    CreateTodo { id, name }: CreateTodo,
) -> Result<Events, Error> {
    Ok(events!(TodoCreated { id, name }))
}

#[derive(Debug, Command)]
pub struct RenameTodo {
    pub id: Id<Todo>,
    pub name: String,
}

#[command_handler]
pub async fn rename_todo(context: &mut TodoContext, command: RenameTodo) -> Result<Events, Error> {
    let todo = context
        .get(command.id)
        .ok_or_else(|| Error(format!("Todo with id {} does not exist", command.id)))?;
    if todo.name != command.name {
        Ok(events!(TodoUpdated::Renamed {
            id: todo.id,
            old_name: todo.name,
            new_name: command.name
        }))
    } else {
        Ok(Default::default())
    }
}

#[derive(Debug, Command)]
pub struct CheckTodo {
    pub id: Id<Todo>,
    pub date: OffsetDateTime,
}

#[command_handler]
pub async fn check_todo(
    context: &mut TodoContext,
    CheckTodo { id, date }: CheckTodo,
) -> Result<Events, Error> {
    let todo = context
        .get(id)
        .ok_or_else(|| Error(format!("Todo with id {} does not exist", id)))?;
    if let TodoState::New = todo.state {
        Ok(events!(TodoUpdated::Done(id, date)))
    } else {
        Ok(Default::default())
    }
}

#[derive(Debug, Command)]
#[presage(name = "archive-done-todo")]
pub struct ArchiveTodo {
    pub id: Id<Todo>,
    pub date: OffsetDateTime,
}

#[command_handler]
pub async fn archive_todo(
    context: &mut TodoContext,
    ArchiveTodo { id, date }: ArchiveTodo,
) -> Result<Events, Error> {
    let todo = context
        .get(id)
        .ok_or_else(|| Error(format!("Todo with id {} does not exist", id)))?;
    if let TodoState::Done { done_date } = todo.state {
        Ok(events!(TodoUpdated::Archived {
            id: todo.id,
            done_date,
            archived_date: date,
        }))
    } else {
        Ok(Default::default())
    }
}

#[derive(Debug, Command)]
pub struct DeleteArchivedTodos;

#[command_handler]
pub async fn delete_archived_todos(
    context: &mut TodoContext,
    _: DeleteArchivedTodos,
) -> Result<Events, Error> {
    let events = context
        .list_archived_todos()
        .into_iter()
        .map(|todo| TodoDeleted(todo.id).serialize())
        .collect::<Result<_, _>>()?;
    Ok(Events(events))
}
