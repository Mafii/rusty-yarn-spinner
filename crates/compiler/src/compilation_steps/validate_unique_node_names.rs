use crate::prelude::generated::yarnspinnerparser::{DialogueContextAttrs, NodeContextAttrs};
use crate::prelude::*;
use antlr_rust::token::Token;
use std::collections::HashMap;

pub fn validate_unique_node_names(mut state: CompilationIntermediate) -> CompilationIntermediate {
    // Ensure that all nodes names in this compilation are unique. Node
    // name uniqueness is important for several processes, so we do this
    // check here.
    let all_nodes = state.parsed_files.iter().flat_map(|file| {
        file.tree
            .node_all()
            .iter()
            .map(|node| (node.clone(), file))
            .collect::<Vec<_>>()
    });

    // Pair up every node with its name, and filter out any that don't
    // have a name
    let nodes_with_names = all_nodes.filter_map(|(node, file)| {
        node.header_all()
            .iter()
            .find(|header| header.header_key.as_ref().unwrap().get_text() == "title")
            .map(|title_header| {
                let title = title_header
                    .header_value
                    .as_ref()
                    .unwrap()
                    .get_text()
                    .to_owned();
                (title, title_header.clone(), file)
            })
    });

    let nodes_by_name = nodes_with_names.fold(
        HashMap::new(),
        |mut map: HashMap<_, Vec<_>>, (name, header_context, file)| {
            map.entry(name).or_default().push((header_context, file));
            map
        },
    );

    // Find groups of nodes with the same name and generate diagnostics
    // for each
    for (name, nodes) in nodes_by_name
        .into_iter()
        .filter(|(_, nodes)| nodes.len() > 1)
    {
        // More than one node has this name! Report an error on both.
        for (header_context, file) in nodes {
            state.diagnostics.push(
                Diagnostic::from_message(format!("More than one node is named {name}",))
                    .with_file_name(file.name.clone())
                    .with_parser_context(header_context.as_ref(), file.tokens()),
            );
        }
    }
    state
}
