//! 唯一对外 FFI 入口：`post_chat`。
//!
//! 调用约定（与其它能力库一致）：
//! `int post_chat(const char *json_in, char *json_out);`
//! `json_out` 由调用方提供 64 KiB 缓冲区。

use serde::Deserialize;
use serde_json::json;
use std::os::raw::{c_char, c_int};

use crate::errcode::{
    self, ERR_CHAT_PROMPT_REQUIRED, ERR_CHAT_REQUEST_FAILED, ERR_SETTING_DB, ERR_SUPPLIER_INVALID,
};
use crate::json::{parse_json, write_error, write_success};
use crate::provider::{self, Message, Options};
use crate::setting::Settings;
use crate::supplier::Supplier;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ThinkingInput {
    Kind(String),
    Object {
        #[serde(rename = "type")]
        kind: String,
    },
}

#[derive(Debug, Deserialize)]
struct ChatReq {
    /// `deepseek` / `siliconflow_t2t`；为空时读取 `ai_supplier`。
    supplier: Option<String>,
    prompt: Option<String>,
    messages: Option<Vec<Message>>,
    system: Option<String>,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f64>,
    top_p: Option<f64>,
    top_k: Option<u32>,
    thinking: Option<ThinkingInput>,
    reasoning_effort: Option<String>,
    /// 业务会话标识，仅回显。
    session_id: Option<String>,
}

/// 统一云端对话入口。
///
/// 请求：
/// ```json
/// {
///   "supplier": "deepseek",           // 可选，缺省读 info.db.setting.ai_supplier
///   "prompt": "你好",                  // 与 messages 至少提供一个
///   "messages": [{"role":"user","content":"你好"}],
///   "system": "...", "model": "...", "max_tokens": 4096,
///   "temperature": 0.7, "top_p": 0.7, "top_k": 50,
///   "thinking": "enabled", "reasoning_effort": "high",
///   "session_id": "..."
/// }
/// ```
///
/// 响应：`{ code: 0, msg, data: { content, id, model, stop_reason, usage, supplier, session_id } }`。
#[unsafe(no_mangle)]
pub extern "C" fn post_chat(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let request: ChatReq = match parse_json(json_in) {
        Ok(request) => request,
        Err(code) => return write_error(json_out, code, errcode::default_message(code)),
    };

    let settings = match Settings::load() {
        Ok(settings) => settings,
        Err(error) => {
            return write_error(
                json_out,
                ERR_SETTING_DB,
                &format!("读取 info.db.setting 失败: {:#}", error),
            )
        }
    };

    let supplier = match request
        .supplier
        .as_deref()
        .map(str::trim)
        .filter(|code| !code.is_empty())
    {
        Some(code) => match Supplier::from_exact_code(code) {
            Some(supplier) => supplier,
            None => {
                return write_error(
                    json_out,
                    ERR_SUPPLIER_INVALID,
                    &format!("未知供应商：{}（可用：deepseek / siliconflow_t2t）", code),
                )
            }
        },
        None => Supplier::from_code(settings.get("ai_supplier")),
    };

    let has_messages = request
        .messages
        .as_ref()
        .map(|messages| !messages.is_empty())
        .unwrap_or(false);
    let has_prompt = request
        .prompt
        .as_ref()
        .map(|prompt| !prompt.trim().is_empty())
        .unwrap_or(false);
    if !has_messages && !has_prompt {
        return write_error(
            json_out,
            ERR_CHAT_PROMPT_REQUIRED,
            "prompt 或 messages 至少提供一个有效内容",
        );
    }

    let options = Options {
        system: request.system,
        model: request.model,
        max_tokens: request.max_tokens,
        temperature: request.temperature,
        top_p: request.top_p,
        top_k: request.top_k,
        thinking: request.thinking.map(|thinking| match thinking {
            ThinkingInput::Kind(kind) => kind,
            ThinkingInput::Object { kind } => kind,
        }),
        reasoning_effort: request.reasoning_effort,
    };

    let result = match supplier {
        Supplier::Deepseek => {
            provider::deepseek::chat(&settings, request.prompt, request.messages, options)
        }
        Supplier::SiliconFlow => {
            provider::siliconflow::chat(&settings, request.prompt, request.messages, options)
        }
    };

    match result {
        Ok(result) => write_success(
            json_out,
            json!({
                "content": result.content,
                "id": result.id,
                "model": result.model,
                "stop_reason": result.stop_reason,
                "usage": result.usage,
                "supplier": supplier.code(),
                "session_id": request.session_id,
            }),
        ),
        Err(error) => write_error(json_out, ERR_CHAT_REQUEST_FAILED, &format!("{:#}", error)),
    }
}
