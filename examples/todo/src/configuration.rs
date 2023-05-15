use presage::Configuration;

use crate::persistence::TodoContext;
use crate::todo::commands::{
    archive_todo, check_todo, create_todo, delete_archived_todos, rename_todo,
};
use crate::todo::views::{
    update_summary_on_todo_created, update_summary_on_todo_deleted, update_summary_on_todo_updated,
};
use crate::Error;

pub fn configuration() -> Configuration<TodoContext, Error> {
    Configuration::new()
        .event_handler(&update_summary_on_todo_created)
        .event_handler(&update_summary_on_todo_updated)
        .event_handler(&update_summary_on_todo_deleted)
        .command_handler(&create_todo)
        .command_handler(&rename_todo)
        .command_handler(&check_todo)
        .command_handler(&archive_todo)
        .command_handler(&delete_archived_todos)
}
