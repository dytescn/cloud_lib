//! FFI JSON 输入 / 输出辅助。
//!
//! 返回值只表示传输层结果：`0` 已写入、`-1` 输出指针为空、`-2` 缓冲区不足。
//! 业务成功 / 失败统一通过响应体的 `code` 表达。

use serde::de::DeserializeOwned;
use serde_json::Value;
use std::ffi::CStr;
use std::os::raw::{c_char, c_int};

use crate::errcode::{self, ERR_JSON_IN_NULL, ERR_JSON_UTF8};

/// 调用方分配的 JSON 输出缓冲区大小（含结尾 NUL）。
pub const OUT_BUF_SIZE: usize = 65536;

/// 写入调用方缓冲区；成功返回 `0`，指针为空返回 `-1`，缓冲区不足返回 `-2`。
pub fn write_json_out(json_out: *mut c_char, value: &Value) -> c_int {
    if json_out.is_null() {
        return -1;
    }
    let text = value.to_string();
    let bytes = text.as_bytes();
    if bytes.len() + 1 > OUT_BUF_SIZE {
        return -2;
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

/// 业务失败也返回 `0`，错误码写在响应体 `code` 中。
pub fn write_error(json_out: *mut c_char, code: i32, msg: &str) -> c_int {
    write_json_out(json_out, &errcode::error(code, Some(msg)))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CStr;

    #[test]
    fn error_writes_body_and_returns_zero() {
        let mut out = [0 as c_char; 256];
        let rc = write_error(out.as_mut_ptr(), 400004, "unknown supplier");
        assert_eq!(rc, 0);

        let text = unsafe { CStr::from_ptr(out.as_ptr()) }.to_str().unwrap();
        let value: Value = serde_json::from_str(text).unwrap();
        assert_eq!(value["code"], 400004);
        assert_eq!(value["msg"], "unknown supplier");
    }

    #[test]
    fn null_out_returns_negative_one() {
        assert_eq!(write_error(std::ptr::null_mut(), 400004, "x"), -1);
    }

    #[test]
    fn oversize_returns_negative_two() {
        let mut out = vec![0 as c_char; OUT_BUF_SIZE];
        let value = serde_json::json!({ "content": "x".repeat(OUT_BUF_SIZE) });
        assert_eq!(write_success(out.as_mut_ptr(), value), -2);
    }
}
