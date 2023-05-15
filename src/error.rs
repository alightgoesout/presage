/// Errors that can occur during the execution of a command by a [CommandBus](crate::CommandBus).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A command handler failed to downcast a [BoxedCommand](crate::BoxedCommand).
    #[error("Could not downcast command to type {0}")]
    CommandDowncastError(&'static str),
    /// An error occurred when serializing or deserializing an [Event](crate::Event).
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    /// A [Command](crate::Command) was dispatched but the command bus does not have a corresponding
    /// [CommandHandler](crate::CommandHandler).
    #[error("Missing command handler for command {0}")]
    MissingCommandHandler(&'static str),
}
