//! 唯一对外 FFI 入口：`chat_t2t`。
//!
//! 调用约定：
//! `int chat_t2t(const char *json_in, char *json_out);`
//! `json_out` 由调用方提供 64 KiB 缓冲区。
//!
//! 当前版本暂不提供 chat 调用：入口保留是为了调用方可以平滑升级，
//! 调用时返回业务错误 `500101`。

use std::os::raw::{c_char, c_int};

use crate::utils::errcode::ERR_CHAT_REQUEST_FAILED;
use crate::utils::json::write_error;

#[unsafe(no_mangle)]
pub extern "C" fn chat_t2t(_json_in: *const c_char, json_out: *mut c_char) -> c_int {
    write_error(json_out, ERR_CHAT_REQUEST_FAILED, "chat 调用尚未实现")
}
