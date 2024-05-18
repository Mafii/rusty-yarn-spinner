//! The main entry point for running a compilation job.

use crate::compilation_steps::*;
use crate::output::*;
use crate::prelude::*;
use crate::string_table_manager::StringTableManager;
use crate::visitors::*;
use crate::Result;
use std::collections::{HashMap, HashSet};

/// Compile Yarn code, as specified by a compilation job.
pub(crate) fn compile(compiler: &Compiler, steps: Vec<&CompilationStep>) -> Result<Compilation> {
    let chars: Vec<Vec<u32>> = compiler
        .files
        .iter()
        .map(|file| {
            // Strip the BOM from the source string if it is present before compiling.
            // Rust does not do this by default
            // https://github.com/rust-lang/rfcs/issues/2428
            let source = match file.source.strip_prefix('\u{feff}') {
                None => file.source.as_str(),
                Some(sanitized_string) => sanitized_string,
            };
            source.chars().map(|c| c as u32).collect()
        })
        .collect();
    let chars: Vec<_> = chars.iter().map(|c| c.as_slice()).collect();
    let initial = CompilationIntermediate::from_job(compiler, chars);
    let intermediate = steps.into_iter().fold(initial, |state, step| {
        if state.early_break {
            state
        } else {
            step(state)
        }
    });
    // Cleaning up diagnostics doesn't change the state but makes sure
    // that diagnostics are unique, there are no errors in the warnings, etc.
    // So we execute it even if we've had early breaks.
    let result = clean_up_diagnostics(intermediate).result.unwrap();
    result
}

pub type CompilationStep = dyn Fn(CompilationIntermediate) -> CompilationIntermediate;

#[derive(Debug)]
pub struct CompilationIntermediate<'input> {
    pub job: &'input Compiler,
    pub file_chars: Vec<&'input [u32]>,
    pub result: Option<Result<Compilation>>,
    /// All variable declarations that we've encountered, PLUS the ones we knew about before
    pub known_variable_declarations: Vec<Declaration>,
    /// All variable declarations that we've encountered during this compilation job
    pub derived_variable_declarations: Vec<Declaration>,
    pub potential_issues: Vec<DeferredTypeDiagnostic>,
    pub parsed_files: Vec<FileParseResult<'input>>,
    pub tracking_nodes: HashSet<String>,
    pub string_table: StringTableManager,
    pub diagnostics: Vec<Diagnostic>,
    pub file_tags: HashMap<String, Vec<String>>,
    pub known_types: KnownTypes,
    pub early_break: bool,
}

impl<'input> CompilationIntermediate<'input> {
    pub(crate) fn from_job(compiler: &'input Compiler, chars: Vec<&'input [u32]>) -> Self {
        Self {
            job: compiler,
            file_chars: chars,
            result: Default::default(),
            known_variable_declarations: Default::default(),
            derived_variable_declarations: Default::default(),
            potential_issues: Default::default(),
            parsed_files: Default::default(),
            tracking_nodes: Default::default(),
            string_table: Default::default(),
            diagnostics: Default::default(),
            file_tags: Default::default(),
            known_types: Default::default(),
            early_break: Default::default(),
        }
    }
}
