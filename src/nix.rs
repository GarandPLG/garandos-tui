use rnix::{NodeOrToken, SyntaxKind, SyntaxNode};

pub fn collect_nix_options_with_path(node: &SyntaxNode, current_path: &str) -> Vec<(String, bool)> {
    let mut result: Vec<(String, bool)> = Vec::new();

    for child in node.children_with_tokens() {
        if let NodeOrToken::Node(child_node) = child {
            match child_node.kind() {
                SyntaxKind::NODE_ATTRPATH_VALUE => {
                    let mut attr_path: String = String::new();
                    let mut value_node: Option<SyntaxNode> = None;

                    for grand in child_node.children_with_tokens() {
                        if let NodeOrToken::Node(grand_node) = grand {
                            match grand_node.kind() {
                                SyntaxKind::NODE_ATTRPATH => {
                                    attr_path = grand_node.text().to_string();
                                }
                                SyntaxKind::NODE_IDENT | SyntaxKind::NODE_LITERAL => {
                                    let text: String = grand_node.text().to_string();
                                    if text == "true" || text == "false" {
                                        value_node = Some(grand_node);
                                    }
                                }
                                SyntaxKind::NODE_ATTR_SET => {
                                    let new_path: String = if current_path.is_empty() {
                                        attr_path.clone()
                                    } else {
                                        format!("{}.{}", current_path, attr_path)
                                    };

                                    result.extend(collect_nix_options_with_path(
                                        &grand_node,
                                        &new_path,
                                    ));
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(value_node) = value_node {
                        let full_path: String = if current_path.is_empty() {
                            attr_path
                        } else {
                            format!("{}.{}", current_path, attr_path)
                        };

                        let value_text: String = value_node.text().to_string();
                        let mut bool_value: bool = false;

                        if value_text == "true" {
                            bool_value = true;
                        }

                        result.push((full_path, bool_value));
                    }
                }

                _ => {
                    result.extend(collect_nix_options_with_path(&child_node, current_path));
                }
            }
        }
    }

    result
}
