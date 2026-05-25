/// 证据链：合并程序切片 + 数据流 + 依赖图分析结果

fn main() {
    let evidence = EvidenceChain {
        finding: "wide-unsafe@data/pointer.rs:12".into(),
        program_slice: vec![
            SliceEntry { file: "data/pointer.rs".into(), line: 8, text: "let ptr = unsafe { alloc(layout) };".into() },
            SliceEntry { file: "data/pointer.rs".into(), line: 10, text: "ptr.write(value);".into() },
            SliceEntry { file: "data/pointer.rs".into(), line: 12, text: "unsafe { transmute::<_, Box<dyn Trait>>(ptr) }".into() },
        ],
        data_flow: vec![
            FlowEntry { var: "layout".into(), from: "alloc::Layout::new::<T>()".into(), line: 3 },
            FlowEntry { var: "ptr".into(), from: "alloc(layout)".into(), line: 8 },
            FlowEntry { var: "value".into(), from: "input".into(), line: 1 },
        ],
        dep_chain: vec![
            "api/handler.rs → data/buffer.rs → data/pointer.rs".into(),
            "service/processor.rs → data/pointer.rs".into(),
        ],
    };

    println!("{}", serde_json::to_string_pretty(&evidence).unwrap());
}

#[derive(serde::Serialize)]
struct EvidenceChain {
    finding: String,
    program_slice: Vec<SliceEntry>,
    data_flow: Vec<FlowEntry>,
    dep_chain: Vec<String>,
}

#[derive(serde::Serialize)]
struct SliceEntry {
    file: String,
    line: usize,
    text: String,
}

#[derive(serde::Serialize)]
struct FlowEntry {
    var: String,
    from: String,
    line: usize,
}
