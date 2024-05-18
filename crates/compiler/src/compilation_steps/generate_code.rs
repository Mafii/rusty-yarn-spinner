use crate::listeners::{CompilerListener, DiagnosticVec};
use crate::prelude::generated::yarnspinnerparser::YarnSpinnerParserTreeWalker;
use crate::prelude::*;
use crate::visitors::KnownTypes;
use crate::Result;
use std::collections::{HashMap, HashSet};

pub fn generate_code(mut state: CompilationIntermediate) -> CompilationIntermediate {
    let has_errors = state.diagnostics.has_errors();
    let results: Vec<_> = if has_errors {
        // We have errors, so we can't safely generate code.
        vec![]
    } else {
        // No errors! Go ahead and generate the code for all parsed files.
        let template = Compilation {
            string_table: state.string_table.0.clone(),
            contains_implicit_string_tags: state.string_table.contains_implicit_string_tags(),
            file_tags: state.file_tags.clone(),
            ..Default::default()
        };
        state
            .parsed_files
            .iter()
            .map(|file| {
                generate_code_for_file(
                    &mut state.tracking_nodes,
                    state.known_types.clone(),
                    template.clone(),
                    file,
                )
            })
            .collect()
    };
    let has_code_generation_errors = results.iter().any(|r| r.is_err());
    let result = if has_errors || has_code_generation_errors {
        let total_diagnostics: Vec<_> = results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .flat_map(|error| error.0.iter())
            .cloned()
            .chain(state.diagnostics.iter().cloned())
            .collect();
        Err(CompilerError(total_diagnostics))
    } else {
        let compilations = results.into_iter().map(|r| r.unwrap());
        Ok(Compilation::combine(
            compilations,
            state.string_table.clone(),
        ))
    };

    state.result = Some(result);
    state
}

fn generate_code_for_file<'a, 'b: 'a, 'input: 'a + 'b>(
    tracking_nodes: &mut HashSet<String>,
    known_types: KnownTypes,
    result_template: Compilation,
    file: &'a FileParseResult<'input>,
) -> Result<Compilation> {
    let compiler_listener = Box::new(CompilerListener::new(
        tracking_nodes.clone(),
        known_types,
        file.clone(),
    ));
    let compiler_tracking_nodes = compiler_listener.tracking_nodes.clone();
    let compiler_diagnostics = compiler_listener.diagnostics.clone();
    let compiler_program = compiler_listener.program.clone();
    let compiler_debug_infos = compiler_listener.debug_infos.clone();

    YarnSpinnerParserTreeWalker::walk(compiler_listener, file.tree.as_ref());

    tracking_nodes.extend(compiler_tracking_nodes.borrow().iter().cloned());

    // Don't attempt to generate debug information if compilation produced errors
    if compiler_diagnostics.borrow().has_errors() {
        Err(CompilerError(compiler_diagnostics.borrow().clone()))
    } else {
        let debug_infos: HashMap<_, _> = compiler_debug_infos
            .borrow()
            .iter()
            .map(|debug_info| (debug_info.node_name.clone(), debug_info.clone()))
            .collect();

        Ok(Compilation {
            program: Some(compiler_program.borrow().clone()),
            warnings: compiler_diagnostics.borrow().clone(),
            debug_info: debug_infos,
            ..result_template
        })
    }
}
