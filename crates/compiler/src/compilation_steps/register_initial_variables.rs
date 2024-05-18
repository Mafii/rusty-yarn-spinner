use crate::prelude::*;
use yarnspinner_core::prelude::*;

pub fn register_initial_variables(mut state: CompilationIntermediate) -> CompilationIntermediate {
    let variables = &mut state.known_variable_declarations;
    let job_variable_declarations = state.job.variable_declarations.clone();
    variables.extend(job_variable_declarations);
    let standard_library_declarations = get_declarations_from_library(&Library::standard_library());
    variables.extend(standard_library_declarations);
    let job_library_declarations = get_declarations_from_library(&state.job.library);
    variables.extend(job_library_declarations);

    state
}
