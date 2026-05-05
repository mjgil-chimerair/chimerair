//! Core Erlang AST types and parsing.
//!
//! Core Erlang is the high-level intermediate representation used by
//! the Erlang compiler. This module provides types for representing
//! Core Erlang abstract syntax trees and a parser for Core Erlang text format.

pub mod ast;
pub mod core_erlang;
pub mod parser;

pub use ast::{
    Apply, Case, Clause, Let, Link, Literal, Module, Monitor, Pattern, PrimOp, Receive, Send,
    Spawn, Try,
};
pub use core_erlang::CoreErlangModule;
pub use parser::{CoreErlangParser, ParseError, ParseResult};
