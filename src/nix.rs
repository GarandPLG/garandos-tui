use rnix::{NodeOrToken, Parse, Root, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, TextSize};
use std::{collections::HashMap, fs, path::PathBuf};

pub struct NixModules {
    pub system_path: PathBuf,
    pub system_ast: Parse<Root>,
    pub system_modules: Vec<ConfigOption>,
    pub home_path: PathBuf,
    pub home_ast: Parse<Root>,
    pub home_modules: Vec<ConfigOption>,
}

#[derive(Clone)]
pub struct ConfigOption {
    pub category: String,
    pub path: String,
    pub value: bool,
    pub source: ConfigSource,
}

#[derive(PartialEq, Clone)]
pub enum ConfigSource {
    System,
    Home,
}

pub fn parse_nix(src: &str) -> Parse<Root> {
    let ast: Parse<Root> = Root::parse(src);
    if !ast.errors().is_empty() {
        eprintln!("Błędy parsowania:");
        for err in ast.errors() {
            eprintln!("  - {}", err);
        }
        panic!("Błąd parsowania pliku .nix");
    }
    ast
}

pub fn build_nix_modules(system_path: PathBuf, home_path: PathBuf) -> NixModules {
    let system_src: String = fs::read_to_string(&system_path).expect("Failed to read system file");
    let home_src: String = fs::read_to_string(&home_path).expect("Failed to read home file");

    let system_ast: Parse<Root> = parse_nix(&system_src);
    let home_ast: Parse<Root> = parse_nix(&home_src);

    let system_modules: Vec<ConfigOption> = collect_nix_options(
        &system_ast.syntax(),
        "",
        "".to_string(),
        ConfigSource::System,
    );
    let home_modules: Vec<ConfigOption> =
        collect_nix_options(&home_ast.syntax(), "", "".to_string(), ConfigSource::Home);

    NixModules {
        system_path,
        system_ast,
        system_modules,
        home_path,
        home_ast,
        home_modules,
    }
}

pub fn get_nix_value_by_path(node: &SyntaxNode, query_path: &str) -> Option<bool> {
    let options: Vec<ConfigOption> =
        collect_nix_options(node, "", "".to_string(), ConfigSource::System);

    let map: HashMap<&str, bool> = options
        .iter()
        .map(|option| (option.path.as_str(), option.value))
        .collect();

    map.get(query_path).copied()
}

pub fn toggle_bool_at_path(node: &SyntaxNode, query_path: &str) -> SyntaxNode {
    let src: String = node.text().to_string();

    fn walk(
        syntax_node: &SyntaxNode,
        cur_path: &str,
        query_path: &str,
    ) -> Option<(rnix::TextRange, bool)> {
        let children: Vec<NodeOrToken<SyntaxNode, SyntaxToken>> =
            syntax_node.children_with_tokens().collect();

        for child in children {
            match child {
                NodeOrToken::Node(attr_val_node)
                    if attr_val_node.kind() == SyntaxKind::NODE_ATTRPATH_VALUE =>
                {
                    let mut attr_path: String = String::new();
                    let mut bool_node: Option<SyntaxNode> = None;

                    for grand in attr_val_node.children_with_tokens() {
                        if let NodeOrToken::Node(grand_node) = grand {
                            match grand_node.kind() {
                                SyntaxKind::NODE_ATTRPATH => {
                                    attr_path = grand_node.text().to_string();
                                }
                                SyntaxKind::NODE_IDENT | SyntaxKind::NODE_LITERAL => {
                                    let txt = grand_node.text();
                                    if txt == "true" || txt == "false" {
                                        bool_node = Some(grand_node);
                                    }
                                }
                                SyntaxKind::NODE_ATTR_SET => {
                                    let next_path: String = if cur_path.is_empty() {
                                        attr_path.clone()
                                    } else {
                                        format!("{}.{}", cur_path, attr_path)
                                    };
                                    if let Some(res) = walk(&grand_node, &next_path, query_path) {
                                        return Some(res);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(bool_node) = bool_node {
                        let full_path: String = if cur_path.is_empty() {
                            attr_path.clone()
                        } else {
                            format!("{}.{}", cur_path, attr_path)
                        };
                        if full_path == query_path {
                            let range: TextRange = bool_node.text_range();
                            let current_is_true: bool = bool_node.text() == "true";
                            return Some((range, current_is_true));
                        }
                    }
                }

                NodeOrToken::Node(inner) => {
                    if let Some(res) = walk(&inner, cur_path, query_path) {
                        return Some(res);
                    }
                }

                _ => {}
            }
        }
        None
    }

    if let Some((range, current_is_true)) = walk(&node, "", query_path) {
        let start: usize = <TextSize as Into<usize>>::into(range.start());
        let end: usize = <TextSize as Into<usize>>::into(range.end());
        let mut new_src: String = src.clone();
        new_src.replace_range(start..end, if current_is_true { "false" } else { "true" });
        Root::parse(&new_src).syntax()
    } else {
        Root::parse(&src).syntax()
    }
}

pub fn collect_nix_options(
    node: &SyntaxNode,
    current_path: &str,
    mut current_category: String,
    current_source: ConfigSource,
) -> Vec<ConfigOption> {
    let mut result: Vec<ConfigOption> = Vec::new();

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
                    current_category = content;
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
                                result.extend(collect_nix_options(
                                    &grand_node,
                                    &new_path,
                                    current_category.clone(),
                                    current_source.clone(),
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
                    let bool_value: bool = value_node.text() == "true";
                    result.push(ConfigOption {
                        category: current_category.clone(),
                        path: full_path,
                        value: bool_value,
                        source: current_source.clone(),
                    });
                }
            }

            NodeOrToken::Node(child_node) => {
                result.extend(collect_nix_options(
                    child_node,
                    current_path,
                    current_category.clone(),
                    current_source.clone(),
                ));
            }

            _ => {}
        }
    }

    result
}
