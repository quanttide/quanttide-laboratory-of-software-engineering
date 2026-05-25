use serde::Serialize;

/// 证据链：合并程序切片、数据流、依赖图三种分析结果
#[derive(Debug, Clone, Serialize)]
pub struct EvidenceChain {
    pub finding_id: String,
    pub program_slice: Vec<SliceEntry>,
    pub data_flow: Vec<FlowEntry>,
    pub dep_slice: Vec<String>,
}

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
pub mod depgraph;
