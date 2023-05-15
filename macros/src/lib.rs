use proc_macro::TokenStream;

mod command;
mod event;
pub(crate) mod utils;

#[proc_macro_derive(Event, attributes(presage))]
pub fn derive_event(event: TokenStream) -> TokenStream {
    event::derive_event::derive_event(event)
}

#[proc_macro_derive(AggregateEvent, attributes(presage, id))]
pub fn derive_aggregate_event(event: TokenStream) -> TokenStream {
    event::derive_aggregate_event::derive_aggregate_event(event)
}

#[proc_macro_attribute]
pub fn event_handler(arguments: TokenStream, handler: TokenStream) -> TokenStream {
    event::event_handler::event_handler(arguments, handler)
}

#[proc_macro_derive(Command, attributes(presage))]
pub fn derive_command(command: TokenStream) -> TokenStream {
    command::derive_command::derive_command(command)
}

#[proc_macro_attribute]
pub fn command_handler(arguments: TokenStream, handler: TokenStream) -> TokenStream {
    command::command_handler::command_handler(arguments, handler)
}
