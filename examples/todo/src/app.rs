use presage::{Command, CommandBus};

use crate::configuration::configuration;
use crate::persistence::TodoContext;
use crate::todo::views::TodosSummary;
use crate::todo::Todo;
use crate::Error;

pub struct TodoApp {
    context: TodoContext,
    command_bus: CommandBus<TodoContext, Error>,
}

impl TodoApp {
    pub fn new() -> Self {
        Self {
            context: Default::default(),
            command_bus: CommandBus::new().configure(configuration()),
        }
    }

    pub async fn execute<C: Command>(&mut self, command: C) -> Result<(), Error> {
        self.command_bus.execute(&mut self.context, command).await
    }

    pub fn summary(&self) -> Result<TodosSummary, Error> {
        Ok(self.context.summary())
    }

    pub fn list_visible_todos(&self) -> Result<Vec<Todo>, Error> {
        Ok(self.context.list_visible_todos())
    }

    pub fn list_archived_todos(&self) -> Result<Vec<Todo>, Error> {
        Ok(self.context.list_archived_todos())
    }
}
