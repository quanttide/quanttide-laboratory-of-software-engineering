use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct SliceEntry {
    pub file: String,
    pub line: usize,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FlowEntry {
    pub var: String,
    pub from: String,
    pub line: usize,
}

pub mod slice;
pub mod dataflow;
pub mod analysis;
