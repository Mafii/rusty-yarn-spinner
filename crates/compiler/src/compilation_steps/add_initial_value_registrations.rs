use crate::prelude::*;
use yarnspinner_core::prelude::*;
use yarnspinner_core::types::{Type, TypeFormat};

pub fn add_initial_value_registrations(
    mut state: CompilationIntermediate,
) -> CompilationIntermediate {
    // Last step: take every variable declaration we found in all
    // of the inputs, and create an initial value registration for
    // it.
    let Ok(compilation) = state.result.as_mut().unwrap().as_mut() else {
        return state;
    };

    let declarations = state
        .known_variable_declarations
        .iter()
        .filter(|decl| !matches!(decl.r#type, Type::Function(_)));

    for declaration in declarations {
        let Some(default_value) = declaration.default_value.clone() else {
            state.diagnostics.push(Diagnostic::from_message(format!(
                "Variable declaration {} (type {}) has a null default value. This is not allowed.",
                declaration.name,
                declaration.r#type.format()
            )));
            continue;
        };
        if let Some(ref mut program) = compilation.program {
            let value = match &declaration.r#type {
                    Type::String => Operand::from(String::from(default_value)),
                    Type::Number => Operand::from(f32::try_from(default_value).unwrap()),
                    Type::Boolean => Operand::from(bool::try_from(default_value).unwrap()),
                    _ => panic!("Cannot create initial value registration for type {}. This is a bug. Please report it at https://github.com/YarnSpinnerTool/YarnSpinner-Rust/issues/new", declaration.r#type.format()),
                };
            program
                .initial_values
                .insert(declaration.name.clone(), value);
        }
    }

    compilation
        .declarations
        .clone_from(&state.derived_variable_declarations);
    state
}
