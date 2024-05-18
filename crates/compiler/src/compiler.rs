//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Compiler/Compiler.cs>
//! and <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Compiler/CompilationJob.cs>

use crate::{compilation_steps::*, prelude::*};
use std::path::Path;
use yarnspinner_core::prelude::*;

mod add_tags_to_lines;
pub(crate) mod antlr_rust_ext;
pub mod run_compilation;
pub(crate) mod utils;

#[allow(missing_docs)]
pub type Result<T> = std::result::Result<T, CompilerError>;

/// An object that contains Yarn source code to compile, and instructions on
/// how to compile it.
///
/// Consume this information using [`Compiler::compile`] to produce a [`Compilation`] result.
///
/// ## Implementation note
///
/// This type is a combination of the original `CompilationStep` and `Compiler` types, optimized for easier, fluent calling.
#[derive(Debug, Clone, PartialEq, Default)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bevy", reflect(Debug, PartialEq))]
#[cfg_attr(
    all(feature = "bevy", feature = "serde"),
    reflect(Serialize, Deserialize)
)]
pub struct Compiler {
    /// The [`File`] structs that represent the content to parse..
    pub files: Vec<File>,

    /// The [`Library`] that contains declarations for functions.
    #[cfg_attr(feature = "bevy", reflect(ignore))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub library: Library,

    /// The types of compilation that the compiler will do.
    pub compilation_type: CompilationType,

    /// The declarations for variables.
    pub variable_declarations: Vec<Declaration>,
}

impl Compiler {
    /// Creates a new [`Compiler`] with the default settings and no files added yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file to the compilation.
    pub fn add_file(&mut self, file: File) -> &mut Self {
        self.files.push(file);
        self
    }

    /// Adds multiple files to the compilation.
    pub fn add_files(&mut self, files: impl IntoIterator<Item = File>) -> &mut Self {
        self.files.extend(files);
        self
    }

    /// Adds a file to the compilation by reading it from disk. Fallible version of [`Compiler::read_file`].
    pub fn try_read_file(&mut self, file_path: impl AsRef<Path>) -> std::io::Result<&mut Self> {
        let file_name = file_path.as_ref().to_string_lossy().to_string();
        let file_content = std::fs::read_to_string(file_path)?;
        self.files.push(File {
            file_name,
            source: file_content,
        });
        Ok(self)
    }

    /// Adds a file to the compilation by reading it from disk. For the fallible version, see [`Compiler::try_read_file`].
    pub fn read_file(&mut self, file_path: impl AsRef<Path>) -> &mut Self {
        self.try_read_file(file_path).unwrap()
    }

    /// Extends the Yarn function library with the given [`Library`]. The standard library is only added if this is called with [`Library::standard_library`].
    pub fn extend_library(&mut self, library: Library) -> &mut Self {
        self.library.extend(library);
        self
    }

    /// Sets the compilation type, which allows premature stopping of the compilation process. By default, this is [`CompilationType::FullCompilation`].
    pub fn with_compilation_type(&mut self, compilation_type: CompilationType) -> &mut Self {
        self.compilation_type = compilation_type;
        self
    }

    /// Adds a variable declaration to the compilation.
    pub fn declare_variable(&mut self, declaration: Declaration) -> &mut Self {
        self.variable_declarations.push(declaration);
        self
    }

    /// Compiles the Yarn files previously added into a [`Compilation`].
    pub fn compile(&self) -> Result<Compilation> {
        run_compilation::compile(self, Self::default_compiler_steps())
    }

    /// Compiles the Yarn files previously added into a [`Compilation`] with custom compilation steps.
    pub fn compile_with_custom_steps(
        &self,
        custom_steps: Vec<&CompilationStep>,
    ) -> Result<Compilation> {
        run_compilation::compile(self, custom_steps)
    }

    /// The default compilation steps that the compiler will use.
    pub fn default_compiler_steps() -> Vec<&'static CompilationStep> {
        vec![
            &register_initial_variables,
            &parse_files,
            &register_strings,
            &validate_unique_node_names,
            &break_on_job_with_only_strings,
            &get_declarations,
            &check_types,
            &find_tracking_nodes,
            &create_declarations_for_tracking_nodes,
            &add_tracking_declarations,
            &resolve_deferred_type_diagnostic,
            &break_on_job_with_only_declarations,
            &generate_code,
            &add_initial_value_registrations,
        ]
    }
}

/// Represents the contents of a file to compile.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bevy", reflect(Debug, PartialEq, Hash))]
#[cfg_attr(
    all(feature = "bevy", feature = "serde"),
    reflect(Serialize, Deserialize)
)]
pub struct File {
    /// The name of the file.
    ///
    /// This may be a full path, or just the filename or anything in
    /// between. This is useful for diagnostics, and for attributing
    /// dialogue lines to their original source files.
    pub file_name: String,

    /// The source code of this file.
    pub source: String,
}

/// The types of compilation that the compiler will do.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "bevy", reflect(Debug, PartialEq, Hash, Default))]
#[cfg_attr(
    all(feature = "bevy", feature = "serde"),
    reflect(Serialize, Deserialize)
)]
pub enum CompilationType {
    /// The compiler will do a full compilation, and generate a [`Program`],
    /// function declaration set, and string table.
    #[default]
    FullCompilation,

    /// The compiler will derive only the variable and function declarations,
    /// and file tags, found in the script.
    DeclarationsOnly,

    /// The compiler will generate a string table only.
    StringsOnly,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn can_call_compile_empty_without_crash() {
        Compiler::new().compile().unwrap();
    }

    #[test]
    fn can_call_compile_file_without_crash() {
        let file = File {
            file_name: "test.yarn".to_string(),
            source: "title: test
---
foo
bar
a {1 + 3} cool expression
==="
            .to_string(),
        };
        Compiler::new().add_file(file).compile().unwrap();
    }
}
