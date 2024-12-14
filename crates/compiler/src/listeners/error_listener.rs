//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Compiler/ErrorListener.cs>

use crate::prelude::generated::yarnspinnerparser::YarnSpinnerParserContextType;
use crate::prelude::generated::yarnspinnerparserlistener::YarnSpinnerParserListener;
use crate::prelude::*;
use antlr_rust::char_stream::InputData;
use antlr_rust::error_listener::ErrorListener;
use antlr_rust::errors::ANTLRError;
use antlr_rust::recognizer::Recognizer;
use antlr_rust::token::Token;
use antlr_rust::token_factory::TokenFactory;
use antlr_rust::tree::ParseTreeListener;
pub use diagnostic::*;
use std::cell::RefCell;
use std::rc::Rc;
use yarnspinner_core::prelude::*;

mod diagnostic;
pub(crate) struct LexerErrorListener {
    pub(crate) diagnostics: RefCell<Vec<Diagnostic>>,
    file_name: String,
}

impl LexerErrorListener {
    pub(crate) fn new(file_name: String) -> Self {
        Self {
            file_name,
            diagnostics: Default::default(),
        }
    }
}

impl<'input, T: Recognizer<'input>> ErrorListener<'input, T> for LexerErrorListener {
    fn syntax_error(
        &self,
        _recognizer: &T,
        _offending_symbol: Option<&<T::TF as TokenFactory<'input>>::Inner>,
        line: isize,
        column: isize,
        msg: &str,
        _error: Option<&ANTLRError>,
    ) {
        let line = (line - 1) as usize;
        let column = column as usize;
        let range = Position {
            line,
            character: column,
        }..Position {
            line,
            character: column + 1,
        };
        self.diagnostics.borrow_mut().push(
            Diagnostic::from_message(msg)
                .with_range(range)
                .with_file_name(&self.file_name),
        );
    }
}

pub(crate) struct ParserErrorListener {
    pub(crate) diagnostics: Rc<RefCell<Vec<Diagnostic>>>,
    /// Not in original implementation, but needed because
    /// [`Token::get_source`] is not implemented by antlr4rust.
    /// So we take the entire file instead of just the file name
    /// and extract the lines ourselves.
    file: File,
}

impl ParserErrorListener {
    pub fn new(file: File) -> Self {
        Self {
            diagnostics: Default::default(),
            file,
        }
    }
}

impl<'input, T: Recognizer<'input>> ErrorListener<'input, T> for ParserErrorListener {
    fn syntax_error(
        &self,
        _recognizer: &T,
        offending_symbol: Option<&<T::TF as TokenFactory<'input>>::Inner>,
        line: isize,
        column: isize,
        msg: &str,
        _error: Option<&ANTLRError>,
    ) {
        let line = line as usize;
        let range = Position {
            line: line.saturating_sub(1),
            character: (column + 1) as usize,
        }..Position {
            line: line.saturating_sub(1),
            character: (column + 1) as usize,
        };
        let mut diagnostic = Diagnostic::from_message(msg)
            .with_file_name(&self.file.file_name)
            .with_range(range);
        if let Some(offending_symbol) = offending_symbol {
            let mut string = String::new();

            // the line with the error on it
            let input = &self.file.source;
            let mut lines = input.lines();
            let error_line = lines.nth(line.saturating_sub(1)).unwrap_or(input);
            string.push_str(error_line);
            string.push('\n');

            // adding indicator symbols pointing out where the error is
            // on the line
            let start = offending_symbol.get_start();
            let stop = offending_symbol.get_stop();
            if start >= 0 && stop >= 0 {
                // the end point of the error in "line space"
                let end = (stop - start) + column + 1;
                for i in 0..end {
                    // move over until we are at the point we need to be
                    if i >= column && i < end {
                        string.push('^');
                    } else {
                        string.push(' ');
                    }
                }
            }

            let line = offending_symbol.get_line_as_usize().saturating_sub(1);
            let column = offending_symbol.get_column_as_usize();
            let length = offending_symbol.get_text().len();
            diagnostic = diagnostic
                .with_context(string)
                .with_start_line(line)
                .with_range(
                    Position {
                        line,
                        character: column,
                    }..Position {
                        line,
                        character: column + length,
                    },
                );
        }
        self.diagnostics.borrow_mut().push(diagnostic);
    }
}

impl ParseTreeListener<'_, YarnSpinnerParserContextType> for ParserErrorListener {}
impl YarnSpinnerParserListener<'_> for ParserErrorListener {}
