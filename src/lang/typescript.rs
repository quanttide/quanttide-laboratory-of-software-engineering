use std::path::Path;

use super::{LanguageParser, ParseResult};

pub struct TypeScriptParser {
    parser: tree_sitter::Parser,
}

impl TypeScriptParser {
    pub fn new() -> Result<Self, String> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            .map_err(|e| format!("设置 TypeScript 语言失败: {}", e))?;
        Ok(Self { parser })
    }
}

impl LanguageParser for TypeScriptParser {
    fn language_name(&self) -> &'static str {
        "TypeScript"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["ts"]
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

pub struct TsxParser {
    parser: tree_sitter::Parser,
}

impl TsxParser {
    pub fn new() -> Result<Self, String> {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_typescript::LANGUAGE_TSX.into())
            .map_err(|e| format!("设置 TSX 语言失败: {}", e))?;
        Ok(Self { parser })
    }
}

impl LanguageParser for TsxParser {
    fn language_name(&self) -> &'static str {
        "TSX"
    }

    fn file_extensions(&self) -> &'static [&'static str] {
        &["tsx"]
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
