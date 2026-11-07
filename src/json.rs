//! FFI JSON 输入 / 输出辅助。

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use crate::errcode::{self, ERR_JSON_IN_NULL, ERR_JSON_OUT_NULL, ERR_JSON_UTF8};

/// 调用方分配的 JSON 输出缓冲区大小（含结尾 NUL）。
pub const OUT_BUF_SIZE: usize = 65536;

/// 写入调用方缓冲区；成功返回 `0`，指针为空返回 `ERR_JSON_OUT_NULL`。
pub fn write_json_out(json_out: *mut c_char, value: &Value) -> c_int {
    if json_out.is_null() {
        return ERR_JSON_OUT_NULL;
    }
    let text = value.to_string();
    let bytes = text.as_bytes();
    if bytes.len() + 1 > OUT_BUF_SIZE {
        let fallback = errcode::error(
            errcode::ERR_JSON_OUT_TOO_LARGE,
            Some("response exceeds 64 KiB"),
        )
        .to_string();
        let fb = fallback.as_bytes();
        if fb.len() + 1 > OUT_BUF_SIZE {
            return errcode::ERR_JSON_OUT_TOO_LARGE;
        }
        unsafe {
            std::ptr::copy_nonoverlapping(fb.as_ptr(), json_out as *mut u8, fb.len());
            *json_out.add(fb.len()) = 0;
        }
        return 0;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), json_out as *mut u8, bytes.len());
        *json_out.add(bytes.len()) = 0;
    }
    0
}

pub fn write_success(json_out: *mut c_char, data: Value) -> c_int {
    write_json_out(json_out, &errcode::success(data))
}

pub fn write_error(json_out: *mut c_char, code: i32, msg: &str) -> c_int {
    let _ = write_json_out(json_out, &errcode::error(code, Some(msg)));
    code
}

/// 从 C 字符串解析 JSON。
pub fn parse_json<T: DeserializeOwned>(json_in: *const c_char) -> Result<T, i32> {
    if json_in.is_null() {
        return Err(ERR_JSON_IN_NULL);
    }
    let body = match unsafe { CStr::from_ptr(json_in) }.to_str() {
        Ok(body) => body,
        Err(_) => return Err(ERR_JSON_UTF8),
    };
    serde_json::from_str(body).map_err(|_| errcode::ERR_JSON_PARSE)
}
