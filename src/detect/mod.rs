use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Finding {
    pub file_path: PathBuf,
    pub line: usize,
    pub column: usize,
    pub severity: Severity,
    pub rule_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Must,
    Should,
    May,
}

pub trait Detector {
    fn rule_id(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn detect(&self, source: &str, tree: &tree_sitter::Tree, file_path: &PathBuf) -> Vec<Finding>;
}

pub mod long_function;
pub mod unsafe_block;
