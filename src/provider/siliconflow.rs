//! SiliconFlow（Anthropic Messages 兼容格式，支持 top_p / top_k）。

use anyhow::Result;
use serde_json::json;

use super::{
    build_messages, insert_system, parse_anthropic_response, send, ChatResult, Message, Options,
};
use crate::setting::Settings;
use crate::supplier::Supplier;

pub fn chat(
    settings: &Settings,
    prompt: Option<String>,
    messages: Option<Vec<Message>>,
    options: Options,
) -> Result<ChatResult> {
    let api_key = settings
        .get("silicon_app_key")
        .ok_or_else(|| anyhow::anyhow!(Supplier::SiliconFlow.missing_token_message(settings)))?
        .to_string();

    let base_url = settings.value_or(
        "silicon_app_base_url",
        Supplier::SiliconFlow.default_base_url(),
    );
    let model = options
        .model
        .or_else(|| settings.get("silicon_app_model").map(str::to_string))
        .unwrap_or_else(|| Supplier::SiliconFlow.default_model().to_string());
    let max_tokens = options
        .max_tokens
        .unwrap_or_else(|| settings.u32_or("silicon_app_max_tokens", 4096));
    let temperature = options
        .temperature
        .unwrap_or_else(|| settings.number_or("silicon_app_temperature", 0.7));
    let top_p = options
        .top_p
        .unwrap_or_else(|| settings.number_or("silicon_app_top_p", 0.7));
    let top_k = options
        .top_k
        .unwrap_or_else(|| settings.u32_or("silicon_app_topk", 50));
    let thinking = options.thinking.unwrap_or_else(|| "disabled".to_string());

    let mut messages = build_messages(prompt, messages)?;
    let system = options.system.or_else(|| {
        settings
            .get("silicon_app_system_prompt")
            .map(str::to_string)
    });
    insert_system(&mut messages, system);

    let request = json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
        "top_p": top_p,
        "top_k": top_k,
        "stream": false,
        "thinking": { "type": thinking },
    });

    let url = build_api_url(&base_url);
    let body = send(
        &url,
        &request,
        ("Authorization", format!("Bearer {}", api_key)),
    )?;
    parse_anthropic_response(&body)
}

pub fn build_api_url(base_url: &str) -> String {
    if base_url.ends_with("/messages") {
        base_url.to_string()
    } else {
        format!("{}/messages", base_url.trim_end_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_urls() {
        assert_eq!(
            build_api_url("https://api.siliconflow.cn/v1"),
            "https://api.siliconflow.cn/v1/messages"
        );
        assert_eq!(
            build_api_url("https://api.siliconflow.cn/v1/"),
            "https://api.siliconflow.cn/v1/messages"
        );
    }
}
