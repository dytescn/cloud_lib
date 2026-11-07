//! 供应商定义。请求里的 `supplier` 与 `info.db.setting.ai_supplier` 都使用这里
//! 的代码；未配置时回退 DeepSeek，保持与 chatllm 原有行为一致。

use crate::setting::Settings;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Supplier {
    Deepseek,
    SiliconFlow,
}

impl Supplier {
    /// 精确解析供应商代码；兼容 `lib` 前缀与 `siliconflow` 别名。
    pub fn from_exact_code(code: &str) -> Option<Self> {
        let code = code.trim();
        let code = code.strip_prefix("lib").unwrap_or(code);
        match code {
            "deepseek" => Some(Supplier::Deepseek),
            "siliconflow_t2t" | "siliconflow" => Some(Supplier::SiliconFlow),
            _ => None,
        }
    }

    /// 空值或未知值回退 `deepseek`。
    pub fn from_code(code: Option<&str>) -> Self {
        code.map(str::trim)
            .filter(|code| !code.is_empty())
            .and_then(Self::from_exact_code)
            .unwrap_or(Supplier::Deepseek)
    }

    pub fn code(self) -> &'static str {
        match self {
            Supplier::Deepseek => "deepseek",
            Supplier::SiliconFlow => "siliconflow_t2t",
        }
    }

    pub fn token_key(self) -> &'static str {
        match self {
            Supplier::Deepseek => "deepseek_app_key",
            Supplier::SiliconFlow => "silicon_app_key",
        }
    }

    pub fn system_prompt_key(self) -> &'static str {
        match self {
            Supplier::Deepseek => "deepseek_app_system_prompt",
            Supplier::SiliconFlow => "silicon_app_system_prompt",
        }
    }

    pub fn default_model(self) -> &'static str {
        match self {
            Supplier::Deepseek => "deepseek-flash",
            Supplier::SiliconFlow => "Qwen/Qwen2.5-7B-Instruct",
        }
    }

    pub fn default_base_url(self) -> &'static str {
        match self {
            Supplier::Deepseek => "https://api.deepseek.com/anthropic",
            Supplier::SiliconFlow => "https://api.siliconflow.cn/v1",
        }
    }

    /// 配置缺失时的错误文案。
    pub fn missing_token_message(self, settings: &Settings) -> String {
        let key = self.token_key();
        if settings.get(key).filter(|v| !v.trim().is_empty()).is_none() {
            format!("AI 配置错误：未配置 {} token：{}", self.code(), key)
        } else {
            format!("AI 配置错误：{} 为空", key)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_codes() {
        assert_eq!(
            Supplier::from_exact_code("deepseek"),
            Some(Supplier::Deepseek)
        );
        assert_eq!(
            Supplier::from_exact_code(" siliconflow_t2t "),
            Some(Supplier::SiliconFlow)
        );
        assert_eq!(
            Supplier::from_exact_code("libsiliconflow_t2t"),
            Some(Supplier::SiliconFlow)
        );
        assert_eq!(Supplier::from_exact_code("openai"), None);
    }

    #[test]
    fn unknown_falls_back_to_deepseek() {
        assert_eq!(Supplier::from_code(None), Supplier::Deepseek);
        assert_eq!(Supplier::from_code(Some("")), Supplier::Deepseek);
        assert_eq!(Supplier::from_code(Some("openai")), Supplier::Deepseek);
        assert_eq!(
            Supplier::from_code(Some("siliconflow_t2t")),
            Supplier::SiliconFlow
        );
    }
}
