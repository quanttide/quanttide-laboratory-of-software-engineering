use std::path::Path;

use super::{LanguageParser, ParseResult};

pub struct DartParser {
    parser: tree_sitter::Parser,
}

impl DartParser {
    pub fn new() -> Result<Self, String> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_dart::LANGUAGE.into())
            .map_err(|e| format!("设置 Dart 语言失败: {}", e))?;
        Ok(Self { parser })
    }
}

impl LanguageParser for DartParser {
    fn language_name(&self) -> &'static str {
        "Dart"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["dart"]
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
