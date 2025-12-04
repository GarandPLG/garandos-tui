use rnix::{NodeOrToken, Root, SyntaxKind, SyntaxNode, SyntaxToken, TextRange, TextSize};
use std::collections::HashMap;

pub struct ConfigOption {
    pub category: String,
    pub path: String,
    pub value: bool,
    pub source: ConfigSource,
}

#[derive(PartialEq)]
pub enum ConfigSource {
    System,
    Home,
}

impl Clone for ConfigSource {
    fn clone(&self) -> Self {
        match self {
            ConfigSource::System => ConfigSource::System,
            ConfigSource::Home => ConfigSource::Home,
        }
    }
}

/// Pobiera wartość logiczną opcji Nix wskazanej pełną ścieżką.
///
/// # Argumenty
///
/// * `node` – korzeń drzewa składniowego Nix (`SyntaxNode`), uzyskany np.
///   z `Root::parse(src).syntax()`.
/// * `query_path` – pełna ścieżka do opcji, np. `services.ssh.enable`.
///
/// # Zwraca
///
/// `Some(true)` lub `Some(false)` jeśli opcja istnieje, w przeciwnym razie `None`.
///
/// # Przykład
///
/// ```
/// use rnix::{Root, SyntaxNode};
/// let src = r#"services.ssh.enable = true;"#;
/// let parse = Root::parse(src);
/// let root: SyntaxNode = parse.node();
///
/// // zwraca Some(true)
/// let val = get_nix_value_by_path(&root, "services.ssh.enable");
/// assert_eq!(val, Some(true));
///
/// // nieistniejąca opcja → None
/// let missing = get_nix_value_by_path(&root, "services.httpd.enable");
/// assert_eq!(missing, None);
/// ```
pub fn get_nix_value_by_path(node: &SyntaxNode, query_path: &str) -> Option<bool> {
    let options: Vec<ConfigOption> = collect_nix_options(node, "", ConfigSource::System);

    let map: HashMap<&str, bool> = options
        .iter()
        .map(|option| (option.path.as_str(), option.value))
        .collect();

    map.get(query_path).copied()
}

/// Przełącza (toggle) wartość logiczną opcji w drzewie Nix określonej przez `query_path`.
///
/// Funkcja odczytuje tekst źródłowy z podanego węzła, znajduje zakres tekstowy
/// odpowiadający wartości boolean i zamienia ją na przeciwną (`true` ↔ `false`).
/// Zwraca nowy węzeł składniowy (`SyntaxNode`) reprezentujący zmodyfikowany
/// kod Nix. Jeśli podana ścieżka nie zostanie znaleziona, zwracany jest węzeł
/// niezmieniony.
///
/// # Argumenty
///
/// * `node` – korzeń drzewa składniowego Nix.
/// * `query_path` – pełna ścieżka do opcji, której wartość ma zostać przełączona.
///
/// # Zwraca
///
/// `SyntaxNode` będący korzeniem drzewa składniowego po zastosowaniu zmiany.
///
/// # Przykład
///
/// ```
/// use rnix::{Root, SyntaxNode};
/// let src = r#"services.ssh.enable = true;"#;
/// let parse = Root::parse(src);
/// let root: SyntaxNode = parse.syntax();
///
/// // przełączamy wartość
/// let new_root = toggle_bool_at_path(&root, "services.ssh.enable");
/// let new_src = new_root.text();
/// assert!(new_src.contains("services.ssh.enable = false"));
/// ```
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

/// Rekurencyjnie przegląda drzewo składniowe Nix i zbiera wszystkie opcje
/// typu `bool`, uwzględniając aktualną ścieżkę oraz kategorię (pobrane z
/// najbliższego blokowego komentarza).
///
/// # Argumenty
///
/// * `node` – bieżący węzeł drzewa składniowego.
/// * `current_path` – dotychczasowa ścieżka do bieżącego węzła (pusty ciąg dla
///   węzła korzenia).
/// * `current_category` – nazwa kategorii pochodząca z najbliższego komentarza
///   blokowego nad węzłem.
///
/// # Zwraca
///
/// `Vec<ConfigOption>` – wektor opcji z kategorią, pełną ścieżką i wartością bool.
///
/// # Przykład
///
/// ```
/// use rnix::{Root, SyntaxNode};
/// // przykładowe źródło Nix
/// let src = r#"
/// /* Services */
/// services.ssh.enable = true;
/// services.httpd.enable = false;
/// "#;
///
/// // parsujemy źródło
/// let ast = Root::parse(src);
///
/// // zbieramy wszystkie opcje bool
/// let options = collect_nix_options(&ast.syntax(), "", ConfigSource::System);
/// assert_eq!(options.len(), 2);
/// assert!(options.iter().any(|option| option.path == "services.ssh.enable" && option.value));
/// assert!(options.iter().any(|option| option.path == "services.httpd.enable" && !option.value));
/// ```
pub fn collect_nix_options(
    node: &SyntaxNode,
    current_path: &str,
    current_source: ConfigSource,
) -> Vec<ConfigOption> {
    let mut result: Vec<ConfigOption> = Vec::new();
    let mut category: String = String::new();

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
                                result.extend(collect_nix_options(
                                    &grand_node,
                                    &new_path,
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
                        category: category.clone(),
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
                    current_source.clone(),
                ));
            }

            _ => {}
        }
    }

    result
}
