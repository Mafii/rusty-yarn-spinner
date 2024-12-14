use crate::parser::generated::yarnspinnerparser::{self, *};
use crate::prelude::generated::yarnspinnerparservisitor::YarnSpinnerParserVisitorCompat;
use antlr_rust::parser_rule_context::ParserRuleContext;
use antlr_rust::token::Token;
use antlr_rust::tree::{ParseTree, ParseTreeVisitorCompat};
use std::collections::HashSet;

#[derive(Clone, Default)]
pub(crate) struct NodeTrackingVisitor {
    pub(crate) tracking_nodes: HashSet<String>,
    pub(crate) ignoring_nodes: HashSet<String>,
    _dummy: Option<String>,
}

impl NodeTrackingVisitor {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}

impl ParseTreeVisitorCompat<'_> for NodeTrackingVisitor {
    type Node = YarnSpinnerParserContextType;
    type Return = Option<String>;

    fn temp_result(&mut self) -> &mut Self::Return {
        &mut self._dummy
    }
}

impl<'input> YarnSpinnerParserVisitorCompat<'input> for NodeTrackingVisitor {
    fn visit_node(&mut self, ctx: &NodeContext<'input>) -> Self::Return {
        let mut title = None;
        let mut tracking = None;
        for header in ctx.header_all() {
            let key = header.header_key.as_ref().unwrap().get_text();
            let value = header
                .header_value
                .as_ref()
                .map(|val| val.get_text().to_owned());
            match key {
                "title" => {
                    title = value;
                }
                "tracking" => {
                    tracking = value;
                }
                _ => {}
            }
        }
        if let Some(title) = title {
            if let Some(tracking) = tracking {
                match tracking.as_str() {
                    "always" => {
                        self.tracking_nodes.insert(title);
                    }
                    "never" => {
                        self.ignoring_nodes.insert(title);
                    }
                    _ => {}
                }
            }
        }
        if let Some(body) = ctx.body() {
            return self.visit(body.as_ref());
        }
        None
    }

    fn visit_valueString(&mut self, ctx: &ValueStringContext<'input>) -> Self::Return {
        ctx.get_token(yarnspinnerparser::STRING, 0)
            .unwrap()
            .get_text()
            .trim_matches('"')
            .to_owned()
            .into()
    }

    fn visit_function_call(&mut self, ctx: &Function_callContext<'input>) -> Self::Return {
        let function_name = ctx
            .get_token(yarnspinnerparser::FUNC_ID, 0)
            .unwrap()
            .get_text();

        if !["visited", "visited_count"].contains(&function_name.as_str()) {
            return None;
        }
        // we aren't bothering to test anything about the value itself
        // if it isn't a static string we'll get back null so can ignore it
        // if the func has more than one parameter later on it will cause an error so again can ignore
        let expression = ctx.expression(0).unwrap();
        let result = self.visit(expression.as_ref());
        if let Some(result) = result {
            self.tracking_nodes.insert(result);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::generated::yarnspinnerparser::YarnSpinnerParser;
    use crate::prelude::*;
    use antlr_rust::common_token_stream::CommonTokenStream;
    use antlr_rust::InputStream;

    #[test]
    fn finds_title_and_tracking_headers() {
        let input = "title: this one is tracking
tracking: always
---
===
title: This one is not tracking
tracking: never
---
===
title: This one says nothing about tracking
---
===
title: This one is tracking, but indecisive
tracking: never
tracking: always
---
===
";
        let result = process_input(input);
        assert_eq!(result.tracking_nodes.len(), 2);
        assert_eq!(result.ignoring_nodes.len(), 1);
        assert!(result.tracking_nodes.contains("this one is tracking"));
        assert!(result
            .tracking_nodes
            .contains("This one is tracking, but indecisive"));
        assert!(result.ignoring_nodes.contains("This one is not tracking"));
    }

    fn process_input(input: &str) -> NodeTrackingVisitor {
        let lexer = YarnSpinnerLexer::new(InputStream::new(input), "input.yarn".to_owned());
        let mut parser = YarnSpinnerParser::new(CommonTokenStream::new(lexer));
        let tree = parser.dialogue().unwrap();
        let mut visitor = NodeTrackingVisitor::new();
        visitor.visit(tree.as_ref());
        visitor
    }
}
