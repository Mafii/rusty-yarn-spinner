//! The compiler components of Yarn Spinner. These mostly follow the same structure as the original Yarn Spinner compiler.
//!
//! You probably don't want to use this crate directly, except if you're coming from another language than Rust and want to call Yarn Spinner via FFI.
//! Otherwise:
//! - If you're a game developer, you'll want to use a crate that is already designed for your game engine of choice,
//! such as [`bevy_yarnspinner`](https://crates.io/crates/bevy_yarnspinner) for the [Bevy engine](https://bevyengine.org/).
//! - If you wish to write an adapter crate for an engine yourself, use the [`yarnspinner`](https://crates.io/crates/yarnspinner) crate.
//!
#![warn(missing_docs, missing_debug_implementations)]

mod collections;
pub mod compilation_steps;
pub(crate) mod compiler;
pub(crate) mod error_strategy;
mod file_parse_result;
pub(crate) mod listeners;
mod output;
pub mod parser;
pub(crate) mod parser_rule_context_ext;
mod string_table_manager;
pub(crate) mod token_ext;
pub(crate) mod visitors;

pub use crate::compiler::run_compilation;
pub use crate::compiler::Result;
pub use antlr_rust::tree::ParseTreeVisitorCompat;

pub mod prelude {
    //! Everything you need to get started with the Yarn Spinner compiler.
    pub(crate) use crate::{
        compiler::antlr_rust_ext::*, compiler::run_compilation::*, compiler::utils::*,
        file_parse_result::*, parser::*, parser_rule_context_ext::*, string_table_manager::*,
        token_ext::*,
    };
    pub use crate::{
        compiler::{CompilationType, Compiler, File},
        listeners::{Diagnostic, DiagnosticSeverity, DiagnosticVec},
        output::*,
    };
    pub(crate) use yarnspinner_core::prelude::*;
}
