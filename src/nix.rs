use rnix::{NodeOrToken, SyntaxKind, SyntaxNode, SyntaxToken};

pub fn collect_nix_options(node: &SyntaxNode) -> Vec<(String, String, bool)> {
    collect_nix_options_with_path(node, "", "")
}

fn collect_nix_options_with_path(
    node: &SyntaxNode,
    current_path: &str,
    current_category: &str,
) -> Vec<(String, String, bool)> {
    let mut result: Vec<(String, String, bool)> = Vec::new();
    let mut category: String = current_category.to_string();

    let children: Vec<NodeOrToken<SyntaxNode, SyntaxToken>> = node.children_with_tokens().collect();

    for i in 0..children.len() {
        match &children[i] {
            NodeOrToken::Token(token) if token.kind() == SyntaxKind::TOKEN_COMMENT => {
                let text: &str = token.text();
                if text.starts_with("/*") && text.ends_with("*/") {
                    let content: String = text
                        .trim_start_matches("/*")
                        .trim_end_matches("*/")
                        .trim()
                        .to_string();
                    category = content;
                }
            }

            NodeOrToken::Node(child_node)
                if child_node.kind() == SyntaxKind::NODE_ATTRPATH_VALUE =>
            {
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
                                    &category,
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

                    result.push((category.clone(), full_path, bool_value));
                }
            }

            NodeOrToken::Node(child_node) => {
                result.extend(collect_nix_options_with_path(
                    child_node,
                    current_path,
                    &category,
                ));
            }

            _ => {}
        }
    }

    result
}
