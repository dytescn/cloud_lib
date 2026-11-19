//! 唯一对外 FFI 入口：`chat_t2t`。
//!
//! 调用约定：
//! `int chat_t2t(const char *json_in, char *json_out);`
//! `json_out` 由调用方提供 1 MiB 缓冲区。
//!
//! 调用流程：
//! 1. 读取 `info.db.setting.ai_supplier`（插件 uuid），未设置直接返回错误；
//! 2. 通过 `plugin_lib` 按 uuid 读取插件信息（name / install_path）；
//! 3. 加载插件动态库，把请求原样转给插件的 `post_chat`，返回插件响应。

use std::os::raw::{c_char, c_int};

use crate::utils::errcode::{
    self, ERR_CHAT_REQUEST_FAILED, ERR_SETTING_DB, ERR_SETTING_MISSING_KEY,
};
use crate::utils::json::{read_input, write_error, write_text};
use crate::utils::plugin;
use crate::utils::setting::Settings;

#[unsafe(no_mangle)]
pub extern "C" fn chat_t2t(json_in: *const c_char, json_out: *mut c_char) -> c_int {
    let request = match read_input(json_in) {
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

    let uuid = match settings.get("ai_supplier") {
        Some(uuid) => uuid.to_string(),
        None => {
            return write_error(
                json_out,
                ERR_SETTING_MISSING_KEY,
                "未设置 ai_supplier（插件 uuid）",
            )
        }
    };

    let info = match plugin::info(&uuid) {
        Ok(info) => info,
        Err(error) => {
            return write_error(
                json_out,
                ERR_CHAT_REQUEST_FAILED,
                &format!("读取插件信息失败: {:#}", error),
            )
        }
    };

    match plugin::call_chat(&info, &request) {
        Ok(response) => write_text(json_out, &response),
        Err(error) => write_error(
            json_out,
            ERR_CHAT_REQUEST_FAILED,
            &format!("调用插件失败: {:#}", error),
        ),
    }
}
