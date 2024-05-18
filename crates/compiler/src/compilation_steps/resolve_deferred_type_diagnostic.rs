use crate::prelude::*;
use std::collections::HashSet;

pub fn resolve_deferred_type_diagnostic(
    mut state: CompilationIntermediate,
) -> CompilationIntermediate {
    let known_declarations: HashSet<_> = state
        .known_variable_declarations
        .iter()
        .map(|decl| &decl.name)
        .collect();

    for deferred_type_diagnostic in &state.potential_issues {
        let resolved = known_declarations.contains(&deferred_type_diagnostic.name);
        if !resolved {
            state
                .diagnostics
                .push(deferred_type_diagnostic.diagnostic.clone())
        }
    }
    state
}
