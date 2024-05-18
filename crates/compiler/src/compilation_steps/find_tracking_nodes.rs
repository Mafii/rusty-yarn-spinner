use crate::prelude::*;
use crate::visitors::NodeTrackingVisitor;
use antlr_rust::tree::ParseTreeVisitorCompat;
use std::collections::HashSet;

pub fn find_tracking_nodes(mut state: CompilationIntermediate) -> CompilationIntermediate {
    // determining the nodes we need to track visits on
    // this needs to be done before we finish up with declarations
    // so that any tracking variables are included in the compiled declarations
    let mut tracking_nodes = HashSet::new();
    let mut ignore_nodes = HashSet::new();
    for file in &state.parsed_files {
        let mut visitor = NodeTrackingVisitor::new();
        visitor.visit(file.tree.as_ref());
        tracking_nodes.extend(visitor.tracking_nodes);
        ignore_nodes.extend(visitor.ignoring_nodes);
    }
    state.tracking_nodes = tracking_nodes.difference(&ignore_nodes).cloned().collect();
    state
}
