use std::collections::HashMap;
use std::ops::{Add, AddAssign};

use crate::{CommandHandler, EventHandler, EventWriter};

/// A configuration for a [CommandBus](crate::CommandBus).
///
/// Implements [Add] and [AddAssign] for composition of multiple configurations.
pub struct Configuration<C, E>
where
    C: 'static,
    E: 'static,
{
    pub(crate) command_handlers: HashMap<&'static str, &'static dyn CommandHandler<C, E>>,
    pub(crate) event_writers: HashMap<&'static str, &'static dyn EventWriter<C, E>>,
    pub(crate) event_handlers: HashMap<&'static str, Vec<&'static dyn EventHandler<C, E>>>,
}

impl<C, E> Configuration<C, E> {
    /// Creates a new empty [Configuration].
    pub fn new() -> Self {
        Self {
            command_handlers: Default::default(),
            event_writers: Default::default(),
            event_handlers: Default::default(),
        }
    }

    /// Adds a new event writer to the configuration. Takes ownership and returns the configuration
    /// to allow chaining.
    pub fn event_writer(mut self, writer: &'static dyn EventWriter<C, E>) -> Self {
        for event_name in writer.event_names() {
            self.event_writers.insert(event_name, writer);
        }
        self
    }

    /// Adds a new event handler to the configuration. Takes ownership and returns the configuration
    /// to allow chaining.
    pub fn event_handler(mut self, handler: &'static dyn EventHandler<C, E>) -> Self {
        for event_name in handler.event_names() {
            self.event_handlers
                .entry(event_name)
                .and_modify(|handlers| handlers.push(handler))
                .or_insert_with(|| vec![handler]);
        }
        self
    }

    /// Adds a new command writer to the configuration. Takes ownership and returns the
    /// configuration to allow chaining.
    pub fn command_handler(mut self, handler: &'static dyn CommandHandler<C, E>) -> Self {
        self.command_handlers
            .insert(handler.command_name(), handler);
        self
    }
}

impl<C, E> Default for Configuration<C, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C, E> Add for Configuration<C, E> {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl<C, E> AddAssign for Configuration<C, E> {
    fn add_assign(&mut self, rhs: Self) {
        self.event_writers.extend(rhs.event_writers);
        self.event_handlers.extend(rhs.event_handlers);
        self.command_handlers.extend(rhs.command_handlers);
    }
}
