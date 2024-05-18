//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Compiler/FileParseResult.cs>

use crate::prelude::{generated::yarnspinnerparser::*, *};
use std::{fmt::Formatter, rc::Rc};

/// Contains the result of parsing a single file of source code.
///
/// This class provides only syntactic information about a parse - that is,
/// it provides access to the parse tree, and the stream of tokens used to
/// produce that parse tree.
#[derive(Clone)]
pub struct FileParseResult<'input> {
    pub name: String,

    pub tree: Rc<DialogueContextAll<'input>>,

    /// This was not in the original, but in Rust we need to actually store
    /// the parser itself somewhere, which is why we store it here.
    /// We also end up leading the `ErrorStrategy` into the public interface, but using generics here makes
    /// the code a lot more complicated without actually providing much benefit.
    pub parser: Rc<ActualYarnSpinnerParser<'input>>,
}

impl<'input> FileParseResult<'input> {
    pub(crate) fn new(
        name: String,
        tree: Rc<DialogueContextAll<'input>>,
        parser: Rc<ActualYarnSpinnerParser<'input>>,
    ) -> Self {
        Self { name, tree, parser }
    }

    pub(crate) fn tokens(&self) -> &ActualTokenStream<'input> {
        &self.parser.input
    }
}

impl<'input> std::fmt::Debug for FileParseResult<'input> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileParseResult")
            .field("name", &self.name)
            .field("tree", &self.tree)
            .finish()
    }
}
