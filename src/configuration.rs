use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::{Add, AddAssign};

use crate::{CommandHandler, EventHandler};

/// A configuration for a [CommandBus](crate::CommandBus).
///
/// Implements [Add] and [AddAssign] for composition of multiple configurations.
pub struct Configuration<C, E>
where
    C: 'static,
    E: 'static,
{
    pub(crate) command_handlers: HashMap<&'static str, &'static dyn CommandHandler<C, E>>,
    pub(crate) event_handlers: HashMap<&'static str, Vec<&'static dyn EventHandler<C, E>>>,
}

impl<C, E> Configuration<C, E> {
    /// Creates a new empty [Configuration].
    pub fn new() -> Self {
        Self {
            command_handlers: Default::default(),
            event_handlers: Default::default(),
        }
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
        for (event, handlers) in rhs.event_handlers {
            match self.event_handlers.entry(event) {
                Entry::Occupied(mut entry) => {
                    entry.get_mut().extend(handlers);
                }
                Entry::Vacant(entry) => {
                    entry.insert(handlers);
                }
            }
        }
        self.command_handlers.extend(rhs.command_handlers);
    }
}

mod test {
    use super::*;
    use crate::{commands, Commands, SerializedEvent};
    use async_trait::async_trait;

    #[test]
    fn test_add_assign() {
        let mut configuration: Configuration<(), ()> =
            Configuration::default().event_handler(&TestEventHandler1);

        configuration += Configuration::default().event_handler(&TestEventHandler2);

        assert_eq!(configuration.event_handlers["test-event"].len(), 2);
    }

    struct TestEventHandler1;

    #[async_trait]
    impl<C, E> EventHandler<C, E> for TestEventHandler1 {
        fn event_names(&self) -> &[&'static str] {
            &["test-event"]
        }

        async fn handle(&self, _: &mut C, _: &SerializedEvent) -> Result<Commands, E> {
            Ok(commands!())
        }
    }

    struct TestEventHandler2;

    #[async_trait]
    impl<C, E> EventHandler<C, E> for TestEventHandler2 {
        fn event_names(&self) -> &[&'static str] {
            &["test-event"]
        }

        async fn handle(&self, _: &mut C, _: &SerializedEvent) -> Result<Commands, E> {
            Ok(commands!())
        }
    }
}
