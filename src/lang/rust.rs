use std::path::Path;

use super::{LanguageParser, ParseResult};

pub struct RustParser {
    parser: tree_sitter::Parser,
}

impl RustParser {
    pub fn new() -> Result<Self, String> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .map_err(|e| format!("设置 Rust 语言失败: {}", e))?;
        Ok(Self { parser })
    }
}

impl LanguageParser for RustParser {
    fn language_name(&self) -> &'static str {
        "Rust"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["rs"]
    }

    fn parse(&mut self, file_path: &Path, source: &str) -> Option<ParseResult> {
        let tree = self.parser.parse(source, None)?;
        Some(ParseResult {
            file_path: file_path.to_string_lossy().to_string(),
            tree,
            source: source.to_string(),
        })
    }
}
