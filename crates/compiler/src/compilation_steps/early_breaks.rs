use crate::prelude::*;

pub fn break_on_job_with_only_strings(
    mut state: CompilationIntermediate,
) -> CompilationIntermediate {
    if state.job.compilation_type == CompilationType::StringsOnly {
        state.result = Some(Ok(Compilation {
            string_table: state.string_table.clone().into(),
            contains_implicit_string_tags: state.string_table.contains_implicit_string_tags(),
            warnings: state.diagnostics.clone(),
            ..Default::default()
        }));
        state.early_break = true;
    }
    state
}

pub fn break_on_job_with_only_declarations(
    mut state: CompilationIntermediate,
) -> CompilationIntermediate {
    if state.job.compilation_type == CompilationType::DeclarationsOnly {
        state.result = Some(Ok(Compilation {
            declarations: state.derived_variable_declarations.clone(),
            warnings: state.diagnostics.clone(),
            file_tags: state.file_tags.clone(),
            ..Default::default()
        }));
        state.early_break = true;
    }
    state
}
