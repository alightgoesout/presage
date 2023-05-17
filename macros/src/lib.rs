use proc_macro::TokenStream;

mod command;
mod event;
pub(crate) mod utils;

/// Derives the [Event](https://docs.rs/presage/latest/presage/trait.Event.html) trait.
///
/// The name of the event is the name of the type converted to kebab case (e.g., `TodoCreated`
/// becomes `todo-created`). To specify another name, use the `#[presage(name = "name")]` attribute.
#[proc_macro_derive(Event, attributes(presage))]
pub fn derive_event(event: TokenStream) -> TokenStream {
    event::derive_event::derive_event(event)
}

/// Derives the [AggregateEvent](https://docs.rs/presage/latest/presage/trait.AggregateEvent.html)
/// trait.
///
/// The name of the event is the name of the type converted to kebab case (e.g., `TodoCreated`
/// becomes `todo-created`). To specify another name, use the `#[presage(name = "name")]` attribute.
///
/// The aggregate type must be provided using the `presage` attribute: `#[presage(Aggregate)]`
/// or `#[presage(aggregate = Aggregate)]`. To extract the aggregate id from the event, an id field
/// is required for a struct or for each variant of an enum. The id field must be annotated with the
/// `#[id]` attribute or its name can be specified on the type with the `presage` attribute :
/// `#[presage(Aggregate, id = id_field)]`
#[proc_macro_derive(AggregateEvent, attributes(presage, id))]
pub fn derive_aggregate_event(event: TokenStream) -> TokenStream {
    event::derive_aggregate_event::derive_aggregate_event(event)
}

/// Creates an [EventHandler](https://docs.rs/presage/latest/presage/trait.EventHandler.html) from a
/// function.
///
/// The function must have two parameters (the context and the handled event) and return a
/// `Result<presage::Commands, _>`. You can use any error type, but if it cannot be extracted from
/// the function signature (e.g., when using a type alias for `Result`), the error type must be
/// specified as argument of the attribute: `#[event_handler(error = MyError)]`.
#[proc_macro_attribute]
pub fn event_handler(arguments: TokenStream, handler: TokenStream) -> TokenStream {
    event::event_handler::event_handler(arguments, handler)
}

/// Derives the [Command](https://docs.rs/presage/latest/presage/trait.Command.html) trait.
///
/// The name of the command is the name of the type converted to kebab case (e.g., `CreateTodo`
/// becomes `create-todo`). To specify another name, use the `#[presage(name = "name")]` attribute.
#[proc_macro_derive(Command, attributes(presage))]
pub fn derive_command(command: TokenStream) -> TokenStream {
    command::derive_command::derive_command(command)
}

/// Creates an [CommandHandler](https://docs.rs/presage/latest/presage/trait.CommandHandler.html)
/// from a function.
///
/// The function must have two parameters (the context and the handled command) and return a
/// `Result<presage::Commands, _>`. You can use any error type, but if it cannot be extracted from
/// the function signature (e.g., when using a type alias for `Result`), the error type must be
/// specified as argument of the attribute: `#[command_handler(error = MyError)]`.
#[proc_macro_attribute]
pub fn command_handler(arguments: TokenStream, handler: TokenStream) -> TokenStream {
    command::command_handler::command_handler(arguments, handler)
}
