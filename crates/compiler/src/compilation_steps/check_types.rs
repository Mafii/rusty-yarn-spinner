use crate::prelude::*;
use crate::visitors::TypeCheckVisitor;
use antlr_rust::tree::ParseTreeVisitorCompat;

pub fn check_types(mut state: CompilationIntermediate) -> CompilationIntermediate {
    for file in &state.parsed_files {
        let mut visitor =
            TypeCheckVisitor::new(state.known_variable_declarations.clone(), file.clone());
        visitor.visit(file.tree.as_ref());
        state
            .known_variable_declarations
            .extend(visitor.new_declarations.clone());
        state
            .derived_variable_declarations
            .extend(visitor.new_declarations);
        state.diagnostics.extend(visitor.diagnostics);
        state.potential_issues.extend(visitor.deferred_types);
        state.known_types.extend(visitor.known_types);
    }
    state
}
