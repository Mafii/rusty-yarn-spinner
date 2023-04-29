use crate::prelude::*;
use antlr_rust::tree::ParseTreeListener;
use rusty_yarn_spinner_core::prelude::*;
mod emit;
use crate::parser::generated::yarnspinnerparser::{
    BodyContext, NodeContext, YarnSpinnerParserContextType,
};
use crate::prelude::generated::yarnspinnerparserlistener::YarnSpinnerParserListener;
pub(crate) use emit::*;

pub(crate) struct CompilerListener<'a, 'b, 'input: 'a + 'b> {
    /// The current node to which instructions are being added.
    pub(crate) current_node: Option<Node>,
    /// The current debug information that describes [`current_node`].
    current_debug_info: DebugInfo,
    pub(crate) debug_infos: Vec<DebugInfo>,

    /// Whether we are currently parsing the
    /// current node as a 'raw text' node, or as a fully syntactic node.
    pub(crate) raw_text_node: bool,

    pub(crate) diagnostics: Vec<Diagnostic>,

    file_parse_result: FileParseResult<'b>,
    tokens: &'a ActualTokenStream<'input>,
    /// The program being generated by the compiler.
    pub(crate) program: Program,
}

impl<'a, 'b, 'input: 'a + 'b> ParseTreeListener<'input, YarnSpinnerParserContextType>
    for CompilerListener<'a, 'b, 'input>
{
}

impl<'a, 'b, 'input: 'a + 'b> YarnSpinnerParserListener<'input>
    for CompilerListener<'a, 'b, 'input>
{
    fn enter_node(&mut self, _ctx: &NodeContext<'input>) {
        // we have found a new node set up the currentNode var ready to
        // hold it and otherwise continue
        self.current_node = Some(Node {
            name: Default::default(),
            instructions: Default::default(),
            labels: Default::default(),
            tags: Default::default(),
            source_text_string_id: Default::default(),
            headers: Default::default(),
        });
        self.current_debug_info = Default::default();
        self.raw_text_node = false;
    }

    fn exit_node(&mut self, ctx: &NodeContext<'input>) {
        let name = &self.current_node.as_ref().unwrap().name.clone();
        if name.is_empty() {
            // We don't have a name for this node. We can't emit code for
            // it.
            self.diagnostics.push(
                Diagnostic::from_message("Missing title header for node")
                    .with_file_name(self.file_parse_result.name.clone())
                    .read_parser_rule_context(ctx, self.tokens),
            );
        } else {
            if !self.program.nodes.contains_key(name) {
                self.program
                    .nodes
                    .insert(name.clone(), self.current_node.clone().unwrap());
            } else {
                // Duplicate node name! We'll have caught this during the
                // declarations pass, so no need to issue an error here.
            }
            self.current_debug_info.node_name = name.clone();
            self.current_debug_info.file_name = self.file_parse_result.name.clone();
            self.debug_infos.push(self.current_debug_info.clone());
        }
        self.current_node = None;
        self.raw_text_node = false;
    }
}
