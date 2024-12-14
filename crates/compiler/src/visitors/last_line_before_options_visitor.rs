//! Adapted from <https://github.com/YarnSpinnerTool/YarnSpinner/blob/da39c7195107d8211f21c263e4084f773b84eaff/YarnSpinner.Compiler/LastLineBeforeOptionsVisitor.cs>

use crate::prelude::generated::yarnspinnerparser::*;
use crate::prelude::generated::yarnspinnerparservisitor::YarnSpinnerParserVisitorCompat;
use crate::prelude::*;
use antlr_rust::tree::ParseTreeVisitorCompat;
use std::rc::Rc;

#[derive(Debug, Clone, Default)]
pub(crate) struct LastLineBeforeOptionsVisitor {
    _dummy: (),
}

impl ParseTreeVisitorCompat<'_> for LastLineBeforeOptionsVisitor {
    type Node = YarnSpinnerParserContextType;
    type Return = ();

    fn temp_result(&mut self) -> &mut Self::Return {
        &mut self._dummy
    }
}

impl<'input> YarnSpinnerParserVisitorCompat<'input> for LastLineBeforeOptionsVisitor {
    // entry point for everything
    // if there are no ifs or options with embedded statements this will be all that is visited
    fn visit_body(&mut self, ctx: &BodyContext<'input>) -> Self::Return {
        self.run_through_statement(&ctx.statement_all());
    }

    // tags this line as being one that is the statement immediately before an option block, does this by adding a #lastline tag onto this line
    // no checking is needed because only lines that are needing to be tagged will be visited, others are skipped.
    // The line is tagged regardless of if there is a #lastline there already
    // technically unnecessary in that case but this feels uncommon enough to not bother edgecasing
    fn visit_line_statement(&mut self, ctx: &Line_statementContext<'input>) -> Self::Return {
        add_hashtag_child(ctx, "lastline")
    }

    // handles the statements inside of an if statement
    // chunks its way through the if, any else-ifs and elses internal block of statements
    fn visit_if_statement(&mut self, ctx: &If_statementContext<'input>) -> Self::Return {
        let statements = ctx.if_clause().unwrap().statement_all();
        self.run_through_statement(&statements);
        for else_if in ctx.else_if_clause_all() {
            let statements = else_if.statement_all();
            self.run_through_statement(&statements);
        }

        if let Some(else_statement) = ctx.else_clause() {
            let statements = else_statement.statement_all();
            self.run_through_statement(&statements);
        }
    }

    // visiting an option
    // basically just run through the statement (if any exist)
    fn visit_shortcut_option_statement(
        &mut self,
        ctx: &Shortcut_option_statementContext<'input>,
    ) -> Self::Return {
        for shortcut in ctx.shortcut_option_all() {
            let statements = shortcut.statement_all();
            self.run_through_statement(&statements);
        }
    }
}

impl LastLineBeforeOptionsVisitor {
    // in the current block of statements finds any lines that immediately follow an option block and visits them for tagging
    // this works by making our way through each and every statement inside of a block performing the following:
    // 1. assume the current statement is an option block
    // 2. assume the statement before it is a line
    // 3. if both of these hold true we have found a line we need to flag as being before options
    // 4. repeat this process until we run out of statements to check
    // this has the potential to have VERY deep call stacks
    fn run_through_statement(&mut self, statements: &[Rc<StatementContextAll>]) {
        for (i, statement) in statements.iter().enumerate() {
            // if we are an if-block we have to visit it in case there are options and lines inside of that
            // once that is done we can move onto the next statement
            if let Some(if_statement) = statement.if_statement() {
                self.visit(if_statement.as_ref());
                continue;
            }

            let Some(shortcut_option_statement) = statement.shortcut_option_statement() else {
                // we aren't an option, keep moving
                continue;
            };
            // we need to visit the option in case it has embedded statements
            self.visit(shortcut_option_statement.as_ref());

            if i == 0 {
                // we are an option BUT there isn't a previous statement
                continue;
            }

            // the statement before us isn't a line, continue
            if let Some(previous) = statements[i - 1].line_statement() {
                // ok now at this point we know the line that needs to be tagged as the last line
                // we do that inside the line visitation
                self.visit(previous.as_ref());
            }
        }
    }
}
