use async_trait::async_trait;
use std::collections::{HashMap, VecDeque};

use crate::{
    BoxedCommand, Command, CommandHandler, Configuration, Error, EventHandler, SerializedEvent,
};

/// Executes a command and handles issued [events](crate::Event).
///
/// Takes a context and a command to execute. The resulting events are persisted, then any matching
/// event handler is executed. Those event handlers can return new commands which are also executed.
/// The process continues as long as new commands are issued.
///
/// Can be created using [new()](CommandBus::new) or the [Default] implementation.
pub struct CommandBus<C, E>
where
    C: 'static,
    E: 'static,
{
    command_handlers: HashMap<&'static str, &'static dyn CommandHandler<C, E>>,
    event_writers: HashMap<&'static str, &'static dyn EventWriter<C, E>>,
    event_handlers: HashMap<&'static str, Vec<&'static dyn EventHandler<C, E>>>,
}

impl<C, E> Default for CommandBus<C, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, E> CommandBus<C, E> {
    /// Creates a new, empty, [CommandBus]
    pub fn new() -> Self {
        Self {
            command_handlers: Default::default(),
            event_writers: Default::default(),
            event_handlers: Default::default(),
        }
    }

    /// Configures the command bus using the specified configuration. Takes ownership of `self` and
    /// returns it to allow chaining.
    ///
    /// # Example
    /// ```
    /// let command_bus = presage::CommandBus::new()
    ///     .configure(
    ///         presage::Configuration::new()
    ///             .event_writer(&some_writer)
    ///             .event_handler(&some_event_handler)
    ///             .command_handler(&some_command_handler)
    ///     );
    /// ```
    pub fn configure(mut self, configuration: Configuration<C, E>) -> Self {
        self.event_writers.extend(configuration.event_writers);
        self.event_handlers.extend(configuration.event_handlers);
        self.command_handlers.extend(configuration.command_handlers);
        self
    }
}

impl<C, E> CommandBus<C, E>
where
    E: From<Error>,
{
    /// Executes a [command](Command) with the provided context. If the execution returns any event,
    /// they are persisted using [event writers](EventWriter), then the corresponding
    /// [event handlers](EventHandler) are executed. If new commands are returned, they are also
    /// executed. The process continues until no more events and commands are issued.
    pub async fn execute<T>(&self, context: &mut C, command: T) -> Result<(), E>
    where
        T: Command,
    {
        let mut commands: VecDeque<BoxedCommand> = VecDeque::from([command.into()]);
        while let Some(command) = commands.pop_front() {
            let events = self
                .get_command_handler(command.name())?
                .handle(context, command)
                .await?;
            for event in events {
                commands.extend(self.handle_event(context, event).await?);
            }
        }
        Ok(())
    }

    fn get_command_handler(
        &self,
        command_name: &'static str,
    ) -> Result<&'static dyn CommandHandler<C, E>, Error> {
        self.command_handlers
            .get(command_name)
            .ok_or(Error::MissingCommandHandler(command_name))
            .map(|handler| *handler)
    }

    async fn handle_event(
        &self,
        context: &mut C,
        event: SerializedEvent,
    ) -> Result<Vec<BoxedCommand>, E> {
        self.write_event(context, &event).await?;
        let mut commands = Vec::new();
        if let Some(handlers) = self.event_handlers.get(event.name()) {
            for handler in handlers {
                commands.extend(handler.handle(context, &event).await?);
            }
        }
        Ok(commands)
    }

    async fn write_event(&self, context: &mut C, event: &SerializedEvent) -> Result<(), E> {
        match self.event_writers.get(event.name()) {
            Some(writer) => writer.write(context, event).await,
            None => {
                log::warn!(
                    "An event of type {} was issued by a command, but no event writer was found to persist it.",
                    event.name(),
                );
                Ok(())
            }
        }
    }
}

impl<C, E> Clone for CommandBus<C, E>
where
    C: 'static,
    E: 'static,
{
    fn clone(&self) -> Self {
        Self {
            command_handlers: self.command_handlers.clone(),
            event_writers: self.event_writers.clone(),
            event_handlers: self.event_handlers.clone(),
        }
    }
}

/// Persists the modifications of events.
///
/// It can either persist the events or persist the results of applying the events.
///
/// # Type arguments
///
/// * `C` - the context for this writer
/// * `E` - the type of errors returned if the writer fails
#[async_trait]
pub trait EventWriter<C, E>: Send + Sync {
    /// The names of the events the writer is able to persist.
    fn event_names(&self) -> &[&'static str];

    /// Writes an event.
    async fn write(&self, context: &mut C, event: &SerializedEvent) -> Result<(), E>;
}
