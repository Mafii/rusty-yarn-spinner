use crate::prelude::*;
use crate::visitors::DeclarationVisitor;
use antlr_rust::tree::ParseTreeVisitorCompat;

pub fn get_declarations(mut state: CompilationIntermediate) -> CompilationIntermediate {
    // Find the variable declarations in these files.
    for file in &state.parsed_files {
        let mut variable_declaration_visitor =
            DeclarationVisitor::new(state.known_variable_declarations.clone(), file.clone());

        variable_declaration_visitor.visit(file.tree.as_ref());

        state
            .known_variable_declarations
            .extend(variable_declaration_visitor.new_declarations.clone());
        state
            .derived_variable_declarations
            .extend(variable_declaration_visitor.new_declarations);

        state
            .diagnostics
            .extend_from_slice(&variable_declaration_visitor.diagnostics);

        state
            .file_tags
            .insert(file.name.clone(), variable_declaration_visitor.file_tags);
    }
    state
}
