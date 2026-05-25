use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// 符号表：符号定义 → 所有引用位置
#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub def_file: PathBuf,
    pub def_line: usize,
    pub refs: Vec<RefLocation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Variable,
    Module,
}

#[derive(Debug, Clone)]
pub struct RefLocation {
    pub file: PathBuf,
    pub line: usize,
}

/// 构建符号表
pub fn build_symbol_table(source: &str, tree: &tree_sitter::Tree, file: &Path) -> SymbolTable {
    let mut symbols = Vec::new();
    let root = tree.root_node();

    // 收集函数定义
    let mut cursor = root.walk();
    loop {
        let n = cursor.node();
        if n.is_named() && n.kind() == "function_item" {
            if let Some(name) = n.child_by_field_name("name")
                .and_then(|nn| nn.utf8_text(source.as_bytes()).ok())
            {
                let def_line = n.start_position().row + 1;
                let mut refs = Vec::new();
                // 在本文件中找所有调用
                let mut c2 = root.walk();
                loop {
                    let n2 = c2.node();
                    if n2.is_named() && n2.kind() == "call_expression" {
                        if let Some(callee) = n2.child_by_field_name("function")
                            .or_else(|| {
                                let mut cc = n2.walk();
                                if cc.goto_first_child() {
                                    loop {
                                        let ch = cc.node();
                                        if ch.is_named() && ch.kind() == "identifier" {
                                            return Some(ch);
                                        }
                                        if !cc.goto_next_sibling() { break; }
                                    }
                                }
                                None
                            })
                            .and_then(|nn| nn.utf8_text(source.as_bytes()).ok())
                        {
                            if callee == name {
                                refs.push(RefLocation {
                                    file: file.to_path_buf(),
                                    line: n2.start_position().row + 1,
                                });
                            }
                        }
                    }
                    if c2.goto_first_child() { continue; }
                    loop { if c2.goto_next_sibling() { break; } if !c2.goto_parent() { break; } }
                }
                symbols.push(Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::Function,
                    def_file: file.to_path_buf(),
                    def_line,
                    refs,
                });
            }
        }
        if cursor.goto_first_child() { continue; }
        loop { if cursor.goto_next_sibling() { break; } if !cursor.goto_parent() { break; } }
    }

    SymbolTable { symbols }
}

/// 重命名符号：生成替换映射
pub fn rename_symbol(table: &SymbolTable, old_name: &str, new_name: &str) -> HashMap<String, String> {
    let mut replacements = HashMap::new();
    for sym in &table.symbols {
        if sym.name == old_name {
            replacements.insert(format!("{}:{}", sym.def_file.display(), sym.def_line), new_name.to_string());
            for rf in &sym.refs {
                replacements.insert(format!("{}:{}", rf.file.display(), rf.line), new_name.to_string());
            }
        }
    }
    replacements
}
