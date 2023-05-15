use presage::{event_handler, Commands};
use std::fmt::{Display, Formatter};

use crate::persistence::TodoContext;
use crate::todo::events::{TodoCreated, TodoDeleted, TodoUpdated};
use crate::Error;

#[derive(Debug, Copy, Clone, Default)]
pub struct TodosSummary {
    new: usize,
    done: usize,
    archived: usize,
}

impl Display for TodosSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Todos [new: {}, done: {}, archived: {}]",
            self.new, self.done, self.archived
        )
    }
}

#[event_handler]
pub async fn update_summary_on_todo_created(
    context: &mut TodoContext,
    _: TodoCreated,
) -> Result<Commands, Error> {
    let mut summary = context.summary();
    summary.new += 1;
    context.save_summary(summary);
    Ok(Commands::default())
}

#[event_handler]
pub async fn update_summary_on_todo_updated(
    context: &mut TodoContext,
    event: TodoUpdated,
) -> Result<Commands, Error> {
    let mut summary = context.summary();

    match event {
        TodoUpdated::Done(..) => {
            summary.new -= 1;
            summary.done += 1;
        }
        TodoUpdated::Archived { .. } => {
            summary.done -= 1;
            summary.archived += 1;
        }
        TodoUpdated::Renamed { .. } => (),
    }

    context.save_summary(summary);
    Ok(Commands::default())
}

#[event_handler]
pub async fn update_summary_on_todo_deleted(
    context: &mut TodoContext,
    _: TodoDeleted,
) -> Result<Commands, Error> {
    let mut summary = context.summary();
    summary.archived -= 1;
    context.save_summary(summary);
    Ok(Commands::default())
}
