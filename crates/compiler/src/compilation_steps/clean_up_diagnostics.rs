use crate::listeners::DiagnosticVec;
use crate::prelude::*;
use std::collections::HashSet;

pub fn clean_up_diagnostics(mut state: CompilationIntermediate) -> CompilationIntermediate {
    let total_diagnostics = if let Some(Ok(compilation)) = state.result.as_ref() {
        compilation
            .warnings
            .iter()
            .cloned()
            .chain(state.diagnostics.iter().cloned())
            .collect()
    } else {
        state.diagnostics.clone()
    };
    let mut unique_diagnostics: HashSet<Diagnostic> = HashSet::from_iter(total_diagnostics.clone());
    let mut ordered_unique_diagnostics = Vec::new();

    // preserve order
    for diagnostic in total_diagnostics {
        if unique_diagnostics.contains(&diagnostic) {
            unique_diagnostics.remove(&diagnostic);
            ordered_unique_diagnostics.push(diagnostic);
        }
    }
    state.diagnostics = ordered_unique_diagnostics;
    if state.diagnostics.has_errors() {
        state.result = Some(Err(CompilerError(state.diagnostics.clone())));
    } else if let Some(Ok(compilation)) = state.result.as_mut() {
        compilation.warnings.clone_from(&state.diagnostics);
    }
    state
}
