use serde::{Deserialize, Serialize};

const DEEPSEEK_API: &str = "https://api.deepseek.com/v1/chat/completions";

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f64,
    max_tokens: u32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

/// LLM 对 finding 的增强结果
#[derive(Debug)]
pub struct LlmEnhancement {
    pub priority: String,
    pub explanation: String,
    pub confidence: String,
}

/// 用 DeepSeek 增强 review finding
pub async fn enhance_finding(code: &str, finding: &str, api_key: &str) -> Result<LlmEnhancement, String> {
    let prompt = format!(
        r#"你是一个代码审查助手。分析以下代码和问题，返回 JSON：

代码：
```
{}
```

问题：{}

请按以下 JSON 格式返回：
{{"priority": "high/medium/low", "explanation": "为什么这是问题（中文）", "confidence": "confirm/dismiss"}}
"#,
        code, finding
    );

    let client = reqwest::Client::new();
    let req = ChatRequest {
        model: "deepseek-chat".into(),
        messages: vec![
            Message { role: "system".into(), content: "你是一个专业的代码审查助手，回答格式为 JSON。".into() },
            Message { role: "user".into(), content: prompt },
        ],
        temperature: 0.1,
        max_tokens: 512,
    };

    let resp = client
        .post(DEEPSEEK_API)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&req)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    let body: ChatResponse = resp
        .json()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let content = body.choices.first()
        .map(|c| &c.message.content)
        .ok_or("LLM 返回空响应")?;

    // 提取 JSON
    let json_str = content
        .trim()
        .strip_prefix("```json").unwrap_or(content)
        .strip_prefix("```").unwrap_or(content)
        .trim()
        .trim_end_matches("```")
        .trim();

    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("解析 LLM JSON 失败: {} (原始响应: {})", e, content))?;

    Ok(LlmEnhancement {
        priority: parsed["priority"].as_str().unwrap_or("medium").to_string(),
        explanation: parsed["explanation"].as_str().unwrap_or("").to_string(),
        confidence: parsed["confidence"].as_str().unwrap_or("confirm").to_string(),
    })
}
