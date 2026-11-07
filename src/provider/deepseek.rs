//! DeepSeek（Anthropic Messages 兼容格式）。

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
        .get("deepseek_app_key")
        .ok_or_else(|| anyhow::anyhow!(Supplier::Deepseek.missing_token_message(settings)))?
        .to_string();

    let base_url = settings.value_or(
        "deepseek_app_base_url",
        Supplier::Deepseek.default_base_url(),
    );
    let model = options
        .model
        .or_else(|| settings.get("deepseek_app_model").map(str::to_string))
        .unwrap_or_else(|| Supplier::Deepseek.default_model().to_string());
    let max_tokens = options
        .max_tokens
        .unwrap_or_else(|| settings.u32_or("deepseek_app_max_tokens", 4096));
    let temperature = options
        .temperature
        .unwrap_or_else(|| settings.number_or("deepseek_app_temperature", 0.7));
    let thinking = options
        .thinking
        .or_else(|| settings.get("deepseek_app_thinking").map(str::to_string))
        .unwrap_or_else(|| "disabled".to_string());
    let reasoning_effort = options
        .reasoning_effort
        .or_else(|| {
            settings
                .get("deepseek_app_reasoning_effort")
                .map(str::to_string)
        })
        .filter(|value| !value.is_empty());

    let mut messages = build_messages(prompt, messages)?;
    let system = options.system.or_else(|| {
        settings
            .get("deepseek_app_system_prompt")
            .map(str::to_string)
    });
    insert_system(&mut messages, system);

    let mut request = json!({
        "model": model,
        "messages": messages,
        "max_tokens": max_tokens,
        "temperature": temperature,
        "stream": false,
        "thinking": { "type": thinking },
    });
    if let Some(effort) = reasoning_effort {
        request["output_config"] = json!({ "effort": effort });
    }

    let url = build_api_url(&base_url);
    let body = send(&url, &request, ("x-api-key", api_key))?;
    parse_anthropic_response(&body)
}

/// `{base_url}/v1/messages`；已经带 `/messages` 时原样使用。
pub fn build_api_url(base_url: &str) -> String {
    if base_url.ends_with("/v1/messages") || base_url.ends_with("/messages") {
        base_url.to_string()
    } else {
        format!("{}/v1/messages", base_url.trim_end_matches('/'))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_urls() {
        assert_eq!(
            build_api_url("https://api.deepseek.com/anthropic"),
            "https://api.deepseek.com/anthropic/v1/messages"
        );
        assert_eq!(
            build_api_url("https://api.deepseek.com/anthropic/"),
            "https://api.deepseek.com/anthropic/v1/messages"
        );
        assert_eq!(
            build_api_url("https://api.deepseek.com/anthropic/v1/messages"),
            "https://api.deepseek.com/anthropic/v1/messages"
        );
    }
}
