//! 通过 `./libary/database` 读取 `info.db` 的 `setting` 表。

use anyhow::{Context, Result};
use libloading::Library;
use serde_json::Value;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::sync::OnceLock;

#[cfg(windows)]
const DB_PATH: &str = "./libary/database.dll";
#[cfg(target_os = "linux")]
const DB_PATH: &str = "./libary/database.so";
#[cfg(target_os = "macos")]
const DB_PATH: &str = "./libary/database.dylib";

/// 逻辑库名；database 内部会拼成 `./database/info.db`。
const DB_NAME: &str = "info";
/// database 1.0.5+ 约定 `json_out` 至少 1 MiB。
const OUT_BUF_SIZE: usize = 1024 * 1024;

type PostDbQueryFn = unsafe extern "C" fn(*const c_char, *mut c_char) -> c_int;

static DB_LIB: OnceLock<Library> = OnceLock::new();
static DB_QUERY_FN: OnceLock<PostDbQueryFn> = OnceLock::new();

fn load_db_query() -> Result<PostDbQueryFn> {
    let lib = unsafe { Library::new(DB_PATH) }.with_context(|| format!("加载 {} 失败", DB_PATH))?;
    let function = unsafe {
        *lib.get::<PostDbQueryFn>(b"db_query")
            .context("dlsym db_query 失败")?
    };
    let _ = DB_LIB.set(lib);
    Ok(function)
}

fn db_query_fn() -> Result<PostDbQueryFn> {
    if let Some(function) = DB_QUERY_FN.get() {
        return Ok(*function);
    }
    let function = load_db_query()?;
    let _ = DB_QUERY_FN.set(function);
    Ok(function)
}

/// 同步执行查询，返回 `data` 数组。
pub fn db_query(sql: &str) -> Result<Vec<Value>> {
    let function = db_query_fn()?;

    let request = serde_json::json!({ "path": DB_NAME, "sql": sql });
    let request = CString::new(request.to_string()).context("请求 JSON 含 \\0")?;
    let mut out = vec![0u8; OUT_BUF_SIZE];

    let ret = unsafe { function(request.as_ptr(), out.as_mut_ptr() as *mut c_char) };
    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_str()
        .context("db_query 响应非 UTF-8")?;
    if text.is_empty() {
        anyhow::bail!("db_query 返回空（ret={}）", ret);
    }

    let value: Value =
        serde_json::from_str(text).with_context(|| format!("db_query 响应解析失败: {}", text))?;
    let code = value.get("code").and_then(Value::as_i64).unwrap_or(-1);
    if code != 0 && code != 200 {
        let msg = value
            .get("msg")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        anyhow::bail!("db_query 错误 (code={}): {}", code, msg);
    }

    Ok(extract_rows(&value))
}

fn extract_rows(value: &Value) -> Vec<Value> {
    let data = value.get("data");
    if let Some(rows) = data.and_then(Value::as_array) {
        return rows.clone();
    }
    for selector in ["list", "rows"] {
        if let Some(rows) = data
            .and_then(|data| data.get(selector))
            .and_then(Value::as_array)
        {
            return rows.clone();
        }
        if let Some(rows) = value.get(selector).and_then(Value::as_array) {
            return rows.clone();
        }
    }
    Vec::new()
}
