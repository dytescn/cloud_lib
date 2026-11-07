//! 统一错误码与响应结构。
//!
//! 约定：
//! - `0` 成功；
//! - `4xxxxx` 客户端错误（输入、参数）；
//! - `5xxxxx` 服务端错误（配置、依赖、上游）。

use serde_json::{json, Value};

pub const OK: i32 = 0;

// 通用输入错误（400xxx）
pub const ERR_JSON_IN_NULL: i32 = 400001;
pub const ERR_JSON_PARSE: i32 = 400002;
pub const ERR_JSON_UTF8: i32 = 400003;
pub const ERR_SUPPLIER_INVALID: i32 = 400004;

// chat（4001xx / 5001xx）
pub const ERR_CHAT_PROMPT_REQUIRED: i32 = 400102;
pub const ERR_CHAT_REQUEST_FAILED: i32 = 500101;

// 配置（5002xx）
pub const ERR_SETTING_DB: i32 = 500201;
pub const ERR_SETTING_MISSING_KEY: i32 = 500202;

// 输出缓冲（5009xx）
pub const ERR_JSON_OUT_NULL: i32 = 500900;
pub const ERR_JSON_OUT_TOO_LARGE: i32 = 500901;

pub fn default_message(code: i32) -> &'static str {
    match code {
        OK => "success",
        ERR_JSON_IN_NULL => "json_in is null",
        ERR_JSON_PARSE => "json parse failed",
        ERR_JSON_UTF8 => "json input is not valid utf-8",
        ERR_SUPPLIER_INVALID => "unknown supplier",
        ERR_CHAT_PROMPT_REQUIRED => "prompt or messages is required",
        ERR_CHAT_REQUEST_FAILED => "chat request failed",
        ERR_SETTING_DB => "read setting failed",
        ERR_SETTING_MISSING_KEY => "setting key is missing",
        ERR_JSON_OUT_NULL => "json_out is null",
        ERR_JSON_OUT_TOO_LARGE => "json output exceeds buffer size",
        _ => "unknown error",
    }
}

pub fn success(data: Value) -> Value {
    json!({ "code": OK, "msg": "success", "data": data })
}

pub fn error(code: i32, msg: Option<&str>) -> Value {
    json!({
        "code": code,
        "msg": msg.unwrap_or_else(|| default_message(code)),
        "data": {}
    })
}
