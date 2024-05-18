use crate::prelude::*;

pub fn parse_files(mut state: CompilationIntermediate) -> CompilationIntermediate {
    for (file, chars) in state.job.files.iter().zip(state.file_chars.iter()) {
        let parse_result = parse_syntax_tree(file, chars, &mut state.diagnostics);
        state.parsed_files.push(parse_result);
    }
    state
}
