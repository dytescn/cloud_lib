//! 两家供应商共用的消息 / 请求参数 / 响应结构。

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod deepseek;
pub mod siliconflow;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }
}

/// 请求级参数；`None` 表示使用 setting 中的默认值。
#[derive(Debug, Default, Clone)]
pub struct Options {
    pub system: Option<String>,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub top_k: Option<u32>,
    pub thinking: Option<String>,
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResult {
    pub content: String,
    pub id: Option<String>,
    pub model: Option<String>,
    pub stop_reason: Option<String>,
    pub usage: Option<Value>,
}

/// 解析 Anthropic Messages 兼容响应中的文本内容。
pub fn parse_anthropic_response(body: &str) -> anyhow::Result<ChatResult> {
    let value: Value = serde_json::from_str(body)
        .map_err(|error| anyhow::anyhow!("响应 JSON 解析失败: {}；原始响应: {}", error, body))?;
    let content = value
        .get("content")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("响应缺少 content 数组: {}", body))?;

    let mut text = String::new();
    for item in content {
        if item.get("type").and_then(Value::as_str) == Some("text") {
            if let Some(part) = item.get("text").and_then(Value::as_str) {
                text.push_str(part);
            }
        }
    }
    if text.is_empty() {
        anyhow::bail!("模型未返回文本内容: {}", body);
    }

    Ok(ChatResult {
        content: text,
        id: value.get("id").and_then(Value::as_str).map(str::to_string),
        model: value
            .get("model")
            .and_then(Value::as_str)
            .map(str::to_string),
        stop_reason: value
            .get("stop_reason")
            .and_then(Value::as_str)
            .map(str::to_string),
        usage: value.get("usage").cloned(),
    })
}

/// 组装 messages：`messages` 优先，否则用 `prompt` 构造一条 user 消息。
pub fn build_messages(
    prompt: Option<String>,
    messages: Option<Vec<Message>>,
) -> anyhow::Result<Vec<Message>> {
    if let Some(messages) = messages.filter(|messages| !messages.is_empty()) {
        return Ok(messages);
    }
    if let Some(prompt) = prompt.filter(|prompt| !prompt.trim().is_empty()) {
        return Ok(vec![Message::user(prompt)]);
    }
    anyhow::bail!("prompt 或 messages 至少提供一个有效内容")
}

/// 首条不是 system 时插入 system。
pub fn insert_system(messages: &mut Vec<Message>, system: Option<String>) {
    if let Some(system) = system.filter(|system| !system.trim().is_empty()) {
        if messages.first().map(|message| message.role.as_str()) != Some("system") {
            messages.insert(0, Message::system(system));
        }
    }
}

pub fn send(url: &str, request: &Value, auth_header: (&str, String)) -> anyhow::Result<String> {
    let response = ureq::post(url)
        .set(auth_header.0, &auth_header.1)
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(120))
        .send_json(request);

    match response {
        Ok(response) => response
            .into_string()
            .map_err(|error| anyhow::anyhow!("读取 API 响应失败: {}", error)),
        Err(ureq::Error::Status(code, response)) => {
            let body = response.into_string().unwrap_or_default();
            anyhow::bail!("API 请求失败\n状态码: {}\n响应: {}", code, body)
        }
        Err(error) => anyhow::bail!("API 请求发送失败: {}", error),
    }
}
