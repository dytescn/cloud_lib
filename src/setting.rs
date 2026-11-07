//! 读取 `info.db.setting`。一次查询取回两个供应商需要的全部配置。

use std::collections::{HashMap, HashSet};

use anyhow::Result;

use crate::db::db_query;

/// 供应商配置键（DeepSeek + SiliconFlow）。
pub const SETTING_KEYS: &[&str] = &[
    "ai_supplier",
    "deepseek_app_key",
    "deepseek_app_base_url",
    "deepseek_app_model",
    "deepseek_app_max_tokens",
    "deepseek_app_temperature",
    "deepseek_app_system_prompt",
    "deepseek_app_thinking",
    "deepseek_app_reasoning_effort",
    "silicon_app_key",
    "silicon_app_base_url",
    "silicon_app_model",
    "silicon_app_max_tokens",
    "silicon_app_temperature",
    "silicon_app_top_p",
    "silicon_app_topk",
    "silicon_app_system_prompt",
];

#[derive(Debug, Default, Clone)]
pub struct Settings {
    values: HashMap<String, String>,
}

impl Settings {
    pub fn load() -> Result<Self> {
        let mut seen = HashSet::new();
        let mut keys = Vec::with_capacity(SETTING_KEYS.len());
        for key in SETTING_KEYS {
            if seen.insert(*key) {
                keys.push(format!("'{}'", key.replace('\'', "''")));
            }
        }
        let sql = format!(
            "SELECT key, value FROM setting WHERE key IN ({})",
            keys.join(", ")
        );
        Ok(Self::from_rows(&db_query(&sql)?))
    }

    pub fn from_rows(rows: &[serde_json::Value]) -> Self {
        let mut values = HashMap::new();
        for row in rows {
            if let (Some(key), Some(value)) = (
                row.get("key").and_then(serde_json::Value::as_str),
                row.get("value").and_then(serde_json::Value::as_str),
            ) {
                values.insert(key.to_string(), value.to_string());
            }
        }
        Self { values }
    }

    #[cfg(test)]
    pub fn from_map(values: HashMap<String, String>) -> Self {
        Self { values }
    }

    /// 返回非空配置值。
    pub fn get(&self, key: &str) -> Option<&str> {
        self.values
            .get(key)
            .map(String::as_str)
            .filter(|value| !value.trim().is_empty())
    }

    pub fn value_or(&self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or(default).to_string()
    }

    pub fn number_or(&self, key: &str, default: f64) -> f64 {
        self.get(key)
            .and_then(|value| value.parse().ok())
            .unwrap_or(default)
    }

    pub fn u32_or(&self, key: &str, default: u32) -> u32 {
        self.number_or(key, f64::from(default)) as u32
    }
}
