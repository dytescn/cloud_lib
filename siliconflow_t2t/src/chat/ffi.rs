// src/chat/ffi.rs
use std::os::raw::{c_char, c_int};

use crate::utils::cjson::{read_text, write_error_json, write_text};
use crate::utils::errcode::{
    self, ERR_CHAT_PROMPT_REQUIRED, ERR_CHAT_REQUEST_FAILED, ERR_SETTING_KEY_MISSING,
};
use crate::utils::setting::{build_params_from_config, load_chat_config_sync};
use crate::utils::siliconflow::siliconflow_chat;

macro_rules! step {
    ($tag:expr, $($arg:tt)*) => {
        eprintln!("[post_chat][{}] {}", $tag, format!($($arg)*));
    };
}

/// 对外 FFI 入口。
///
/// - 输入：`json_in` 是一段纯字符串 prompt；
/// - 成功：`json_out` 写入模型回复文本；
/// - 失败：`json_out` 写入 `{code,msg,data}` 错误 JSON。
///
/// 请求参数全部来自 `info.db.setting`；任一配置缺失或无效时返回 `500202`。
#[no_mangle]
pub extern "C" fn post_chat(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let prompt = match read_text(json_in) {
        Ok(prompt) if !prompt.trim().is_empty() => prompt,
        Ok(_) => return write_error_json(json_out, ERR_CHAT_PROMPT_REQUIRED, "prompt 不能为空"),
        Err(code) => return write_error_json(json_out, code, errcode::default_message(code)),
    };

    let cfg = match load_chat_config_sync() {
        Ok(cfg) => cfg,
        Err(error) => {
            step!("setting", "读取失败: {:#}", error);
            return write_error_json(
                json_out,
                ERR_SETTING_KEY_MISSING,
                &format!("配置读取失败: {:#}", error),
            );
        }
    };

    step!("chat", "prompt 长度 {}", prompt.len());
    let params = build_params_from_config(&cfg, prompt);

    match siliconflow_chat(&cfg.api_key, &cfg.base_url, &params) {
        Ok(result) => {
            step!("chat", "成功，回复长度 {}", result.content.len());
            write_text(json_out, &result.content)
        }
        Err(error) => {
            step!("chat", "失败: {:?}", error);
            let msg = format!(
                "{}: {:#}",
                errcode::default_message(ERR_CHAT_REQUEST_FAILED),
                error
            );
            write_error_json(json_out, ERR_CHAT_REQUEST_FAILED, &msg)
        }
    }
}
