//! # Présage \pʁe.zaʒ\
//!
//! Présage is a Rust lightweight library for designing event-based systems.
//!
//! ## Concepts
//!
//! In an event-based systems, anything that can happen is modeled with an [Event]. Business
//! entities are designed with [aggregates](Aggregate), that are atomically modified with
//! [events](AggregateEvent).
//!
//! Présage is freely inspired by concepts like domain-driven design or command-query responsibility
//! segregation, but tries to be agnostic by making as few assumptions as possible. Specifically, it
//! does not tie you to any persistence approach.
//!
//! ## Command bus
//!
//! The command bus is the main entry point when using présage. It takes a context and a
//! [command](Command) to execute. The resulting events are persisted, then any matching event
//! handler is executed. Those event handlers can return new commands which are also executed. The
//! process continues as long as events and commands are issued.
//!
//! ## Context
//!
//! [Command handlers](CommandHandler) and [event handlers](EventHandler) are executed within a
//! mutable context. This context is specific to your application and contains whatever is necessary
//! for the execution of the handlers. For instance, it can contain a connection to a database.
//!
//! ## Features
//!
//! The `derive` feature, which is enabled by default, provides derive macros for [Event],
//! [AggregateEvent] and [Command], as well as attribute macros to easily create
//! [command handlers](CommandHandler) and [event handlers](EventHandler).

#![forbid(unsafe_code)]
#![deny(missing_docs)]
#![cfg_attr(__docs, feature(doc_auto_cfg))]

mod aggregate;
mod command;
mod command_bus;
mod configuration;
mod error;
mod event;

pub use aggregate::{Aggregate, Id};
pub use command::{BoxedCommand, Command, CommandHandler, Commands};
pub use command_bus::{CommandBus, EventWriter};
pub use configuration::Configuration;
pub use error::Error;
pub use event::{AggregateEvent, Event, EventHandler, Events, SerializedEvent};

#[cfg(feature = "derive")]
pub use presage_macros::{command_handler, event_handler, AggregateEvent, Command, Event};

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use async_trait::async_trait;
