use crate::prelude::*;
use crate::visitors::{LastLineBeforeOptionsVisitor, StringTableGeneratorVisitor};
use antlr_rust::tree::ParseTreeVisitorCompat;

pub fn register_strings(mut state: CompilationIntermediate) -> CompilationIntermediate {
    // First pass: parse all files, generate their syntax trees,
    // and figure out what variables they've declared
    for file in &state.parsed_files {
        // ok now we will add in our lastline tags
        // we do this BEFORE we build our strings table otherwise the tags will get missed
        // this should probably be a flag instead of every time though
        let mut last_line_tagger = LastLineBeforeOptionsVisitor::default();
        last_line_tagger.visit(file.tree.as_ref());

        let mut visitor =
            StringTableGeneratorVisitor::new(state.string_table.clone(), file.clone());
        visitor.visit(file.tree.as_ref());
        state.diagnostics.extend(visitor.diagnostics);
        state.string_table.extend(visitor.string_table_manager);
    }

    state
}
