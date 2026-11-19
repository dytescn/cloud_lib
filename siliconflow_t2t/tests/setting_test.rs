use std::collections::HashMap;

use siliconflow_t2t::utils::setting::{build_chat_config, build_params_from_config, ChatConfig};

#[test]
fn test_build_chat_config_full() {
    let mut m = HashMap::new();
    for (key, value) in [
        ("silicon_app_key", "sk-test"),
        ("silicon_app_base_url", "https://api.siliconflow.cn/v1"),
        ("silicon_app_model", "Qwen/Qwen2.5-7B-Instruct"),
        ("silicon_app_max_tokens", "4096"),
        ("silicon_app_temperature", "0.7"),
        ("silicon_app_top_p", "0.7"),
        ("silicon_app_topk", "40"),
        ("silicon_app_system_prompt", "你是助手"),
    ] {
        m.insert(key.to_string(), value.to_string());
    }

    let cfg = build_chat_config(&m).unwrap();
    assert_eq!(cfg.api_key, "sk-test");
    assert_eq!(cfg.system_prompt, Some("你是助手".to_string()));
    assert_eq!(cfg.max_tokens, 4096);
    assert_eq!(cfg.top_p, 0.7);
    assert_eq!(cfg.top_k, 40);
}

#[test]
fn test_build_chat_config_requires_every_field() {
    let mut m = HashMap::new();
    for (key, value) in [
        ("silicon_app_key", "sk-test"),
        ("silicon_app_base_url", "https://api.siliconflow.cn/v1"),
        ("silicon_app_model", "Qwen/Qwen2.5-7B-Instruct"),
        ("silicon_app_max_tokens", "4096"),
        ("silicon_app_temperature", "0.7"),
        ("silicon_app_top_p", "0.7"),
        ("silicon_app_topk", "40"),
        ("silicon_app_system_prompt", "你是助手"),
    ] {
        m.insert(key.to_string(), value.to_string());
    }

    for key in [
        "silicon_app_key",
        "silicon_app_base_url",
        "silicon_app_model",
        "silicon_app_max_tokens",
        "silicon_app_temperature",
        "silicon_app_top_p",
        "silicon_app_topk",
        "silicon_app_system_prompt",
    ] {
        let mut missing = m.clone();
        missing.remove(key);
        assert!(build_chat_config(&missing).is_err(), "{key} 缺失时应报错");
    }
}

#[test]
fn test_build_params_with_system() {
    let cfg = ChatConfig {
        api_key: "k".into(),
        base_url: "u".into(),
        model: "m".into(),
        max_tokens: 100,
        temperature: 0.5,
        top_p: 0.9,
        top_k: 10,
        system_prompt: Some("你是助手".into()),
    };

    let p = build_params_from_config(&cfg, "你好".into());
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
        top_p: 0.9,
        top_k: 10,
        system_prompt: None,
    };

    let p = build_params_from_config(&cfg, "你好".into());
    assert_eq!(p.messages.len(), 1);
    assert_eq!(p.messages[0].role, "user");
}
