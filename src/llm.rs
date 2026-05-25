use serde::{Deserialize, Serialize};

const DEEPSEEK_API: &str = "https://api.deepseek.com/v1/chat/completions";
const VAULT_ADDR: &str = "http://127.0.0.1:8200";

/// 从 Vault 读取 DeepSeek API Key
pub async fn get_api_key_from_vault() -> Result<String, String> {
    let token = std::env::var("VAULT_TOKEN")
        .map_err(|_| "VAULT_TOKEN 环境变量未设置".to_string())?;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("{}/v1/secret/data/deepseek", VAULT_ADDR))
        .header("X-Vault-Token", &token)
        .send()
        .await
        .map_err(|e| format!("Vault 请求失败: {}", e))?;

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Vault 响应解析失败: {}", e))?;

    let key = body["data"]["data"]["apiKey"]
        .as_str()
        .ok_or("Vault 中未找到 apiKey 字段")?
        .to_string();

    Ok(key)
}

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

/// 计算 reflect 输出的置信度（确定性计算，非 LLM 自评）
/// 基于证据锚定率：行号引用数 + 变量名引用数
pub fn compute_confidence(text: &str) -> &'static str {
    let line_refs = count_pattern(text, &["L", "行", "line"]);
    let var_refs = count_pattern(text, &["parts", "price", "qty", "trim", "parse",
        "total", "sum", "v", "name", "threshold", "items", "item"]);
    let total = line_refs + var_refs;

    if total >= 3 { "high" }
    else if total >= 1 { "medium" }
    else { "low" }
}

fn count_pattern(text: &str, patterns: &[&str]) -> usize {
    let mut count = 0;
    for p in patterns {
        let mut start = 0;
        while let Some(pos) = text[start..].find(p) {
            count += 1;
            start += pos + p.len();
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_confidence_high() {
        let t = "L3 的 price 解析和 L7 的 qty 解析用了相同模式";
        assert_eq!(compute_confidence(t), "high");
    }

    #[test]
    fn test_compute_confidence_medium() {
        let t = "L8 的数组访问未检查长度";
        assert_eq!(compute_confidence(t), "medium");
    }

    #[test]
    fn test_compute_confidence_low() {
        let t = "职责不够单一，建议架构分层";
        assert_eq!(compute_confidence(t), "low");
    }
}
