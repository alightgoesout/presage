use async_trait::async_trait;
use std::any::{type_name, Any};
use std::fmt::Debug;

use crate::{Error, Events};

/// A request to modify the system.
///
/// A corresponding [command handler](CommandHandler) must be defined.
///
/// # Associated constant
///
/// * [NAME](Self::NAME) - the unique name of the command
///
/// # Example
///
/// ```
/// # use presage::Id;
/// #
/// #[derive(Debug)]
/// pub struct CreateTodo {
///     pub id: Id<Todo>,
///     pub name: String,
/// }
///
/// impl presage::Command for CreateTodo {
///     const NAME: &'static str = "create-todo";
/// }
/// ```
pub trait Command: Sized + Send + Sync + 'static {
    /// The name of the command. Must be unique.
    const NAME: &'static str;
}

/// A command that has been boxed to be dispatched.
///
/// Can be created from a [Command].
#[derive(Debug)]
pub struct BoxedCommand {
    name: &'static str,
    command: Box<dyn Any + Send + Sync>,
}

impl BoxedCommand {
    /// Returns the name of the boxed command.
    pub fn name(&self) -> &'static str {
        self.name
    }

    /// Tries to downcast the boxed command to a concrete [Command] implementation.
    pub fn downcast<C: Command>(self) -> Result<C, Error> {
        self.command
            .downcast()
            .map(|command| *command)
            .map_err(|_| Error::CommandDowncastError(type_name::<C>()))
    }
}

impl<C: Command> From<C> for BoxedCommand {
    fn from(command: C) -> Self {
        BoxedCommand {
            name: C::NAME,
            command: Box::new(command),
        }
    }
}

/// Wrapper for a [Vec] of [boxed commands](BoxedCommand).
#[derive(Debug, Default)]
#[repr(transparent)]
pub struct Commands(pub Vec<BoxedCommand>);

impl Commands {
    /// Creates an empty [Commands].
    pub fn new() -> Self {
        Self::default()
    }

    /// Boxes a command and adds it to the wrapped [Vec].
    pub fn add(&mut self, command: impl Command) {
        self.0.push(command.into())
    }
}

impl FromIterator<BoxedCommand> for Commands {
    fn from_iter<T: IntoIterator<Item = BoxedCommand>>(iter: T) -> Self {
        Commands(iter.into_iter().collect())
    }
}

impl IntoIterator for Commands {
    type Item = BoxedCommand;
    type IntoIter = <Vec<BoxedCommand> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Creates a new [Commands] containing the commands passed as arguments.
#[macro_export]
macro_rules! commands {
    ($($commands: expr),* $(,)?) => {
        $crate::Commands(vec![$($commands.into()),*])
    }
}

/// Handles a command and produces [events](crate::Event).
///
/// # Type arguments
///
/// * `C` - the context for this handler
/// * `E` - the type of errors returned if the handler fails
#[async_trait]
pub trait CommandHandler<C, E>: Send + Sync {
    /// The name of the handled command.
    fn command_name(&self) -> &'static str;

    /// Executes a command, with the given context.
    async fn handle(&self, context: &mut C, command: BoxedCommand) -> Result<Events, E>;
}
