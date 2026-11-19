// tests/setting_test.rs —— DeepSeek 配置与消息组装测试
use std::collections::HashMap;

use deepseek::utils::setting::{build_chat_config, build_params_from_config, ChatConfig};

#[test]
fn test_build_chat_config_full() {
    let mut m = HashMap::new();
    for (key, value) in [
        ("deepseek_app_key", "sk-test"),
        ("deepseek_app_base_url", "https://api.deepseek.com/anthropic"),
        ("deepseek_app_model", "deepseek-flash"),
        ("deepseek_app_max_tokens", "4096"),
        ("deepseek_app_temperature", "0.7"),
        ("deepseek_app_system_prompt", "你是助手"),
        ("deepseek_app_thinking", "disabled"),
        ("deepseek_app_reasoning_effort", "high"),
    ] {
        m.insert(key.to_string(), value.to_string());
    }

    let cfg = build_chat_config(&m).expect("配置解析失败");
    assert_eq!(cfg.api_key, "sk-test");
    assert_eq!(cfg.system_prompt, Some("你是助手".to_string()));
    assert_eq!(cfg.model, "deepseek-flash");
    assert_eq!(cfg.max_tokens, 4096);
    assert_eq!(cfg.temperature, 0.7);
    assert_eq!(cfg.thinking.kind, "disabled");
    assert_eq!(cfg.reasoning_effort, Some("high".to_string()));
}

#[test]
fn test_build_chat_config_requires_every_field() {
    let mut m = HashMap::new();
    for (key, value) in [
        ("deepseek_app_key", "sk-test"),
        ("deepseek_app_base_url", "https://api.deepseek.com/anthropic"),
        ("deepseek_app_model", "deepseek-flash"),
        ("deepseek_app_max_tokens", "4096"),
        ("deepseek_app_temperature", "0.7"),
        ("deepseek_app_system_prompt", "你是助手"),
        ("deepseek_app_thinking", "disabled"),
        ("deepseek_app_reasoning_effort", "high"),
    ] {
        m.insert(key.to_string(), value.to_string());
    }

    for key in [
        "deepseek_app_key",
        "deepseek_app_base_url",
        "deepseek_app_model",
        "deepseek_app_max_tokens",
        "deepseek_app_temperature",
        "deepseek_app_system_prompt",
        "deepseek_app_thinking",
        "deepseek_app_reasoning_effort",
    ] {
        let mut missing = m.clone();
        missing.remove(key);
        assert!(build_chat_config(&missing).is_err(), "{key} 缺失时应报错");
    }
}

#[test]
fn test_build_chat_config_requires_key() {
    let m = HashMap::new();
    assert!(build_chat_config(&m).is_err());
}

#[test]
fn test_build_params_with_system() {
    let cfg = ChatConfig {
        api_key: "k".into(),
        base_url: "u".into(),
        model: "m".into(),
        max_tokens: 100,
        temperature: 0.5,
        system_prompt: Some("你是助手".into()),
        thinking: deepseek::utils::deepseek::Thinking::disabled(),
        reasoning_effort: None,
    };

    let p = build_params_from_config(&cfg, "你好".into(), cfg.system_prompt.clone());
    assert_eq!(p.messages.len(), 2);
    assert_eq!(p.messages[0].role, "system");
    assert_eq!(p.messages[1].role, "user");
}

#[test]
fn test_build_params_without_system() {
    let cfg = ChatConfig {
        api_key: "k".into(),
        base_url: "u".into(),
        model: "m".into(),
        max_tokens: 100,
        temperature: 0.5,
        system_prompt: None,
        thinking: deepseek::utils::deepseek::Thinking::disabled(),
        reasoning_effort: None,
    };

    let p = build_params_from_config(&cfg, "你好".into(), None);
    assert_eq!(p.messages.len(), 1);
    assert_eq!(p.messages[0].role, "user");
}
