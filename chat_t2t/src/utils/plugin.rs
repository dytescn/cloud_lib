//! 插件查询与调用。
//!
//! 通过 `plugin_lib` 的 `get_plugins` 按 uuid 读取插件信息，再按
//! `install_path` 定位插件动态库并调用其 `post_chat` 入口。

use anyhow::{Context, Result};
use libloading::Library;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};

#[cfg(windows)]
const DYLIB_EXT: &str = "dll";
#[cfg(target_os = "linux")]
const DYLIB_EXT: &str = "so";
#[cfg(target_os = "macos")]
const DYLIB_EXT: &str = "dylib";

#[cfg(windows)]
const PLUGIN_LIB_PATH: &str = "./libary/plugin_lib.dll";
#[cfg(target_os = "linux")]
const PLUGIN_LIB_PATH: &str = "./libary/plugin_lib.so";
#[cfg(target_os = "macos")]
const PLUGIN_LIB_PATH: &str = "./libary/plugin_lib.dylib";

/// 库间调用（`plugin_lib` / 插件）按 1 MiB 分配输出缓冲区。
const OUT_BUF_SIZE: usize = 1024 * 1024;

/// 插件聊天入口符号。
const PLUGIN_ENTRY: &[u8] = b"post_chat";

type FfiFn = unsafe extern "C" fn(*const c_char, *mut c_char) -> c_int;

/// 插件信息（来自 `plugins` 表）。
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub uuid: String,
    pub name: String,
    pub install_path: String,
}

static PLUGIN_LIB_FN: OnceLock<FfiFn> = OnceLock::new();
static PLUGIN_LIB: OnceLock<Library> = OnceLock::new();
static PLUGIN_LIBS: OnceLock<Mutex<HashMap<String, Arc<Library>>>> = OnceLock::new();

fn plugin_lib_fn() -> Result<FfiFn> {
    if let Some(function) = PLUGIN_LIB_FN.get() {
        return Ok(*function);
    }
    let lib = unsafe { Library::new(PLUGIN_LIB_PATH) }
        .with_context(|| format!("加载 {} 失败", PLUGIN_LIB_PATH))?;
    let function = unsafe {
        *lib.get::<FfiFn>(b"get_plugins")
            .context("dlsym get_plugins 失败")?
    };
    let _ = PLUGIN_LIB.set(lib);
    let _ = PLUGIN_LIB_FN.set(function);
    Ok(function)
}

/// 调用 `plugin_lib` 的 `get_plugins`，返回 `data`。
fn plugin_info(uuid: &str) -> Result<Value> {
    let function = plugin_lib_fn()?;
    let request = json!({ "uuid": uuid });
    let request = CString::new(request.to_string()).context("请求 JSON 含 \\0")?;
    let mut out = vec![0u8; OUT_BUF_SIZE];

    let ret = unsafe { function(request.as_ptr(), out.as_mut_ptr() as *mut c_char) };
    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_str()
        .context("get_plugins 响应非 UTF-8")?;
    if text.is_empty() {
        anyhow::bail!("get_plugins 返回空（ret={}）", ret);
    }

    let value: Value = serde_json::from_str(text)
        .with_context(|| format!("get_plugins 响应解析失败: {}", text))?;
    let code = value.get("code").and_then(Value::as_i64).unwrap_or(-1);
    if code != 0 && code != 200 {
        let msg = value
            .get("msg")
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        anyhow::bail!("get_plugins 错误 (code={}): {}", code, msg);
    }

    value
        .get("data")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("get_plugins 响应缺少 data: {}", text))
}

/// 按插件 uuid 读取插件信息。
pub fn info(uuid: &str) -> Result<PluginInfo> {
    let value = plugin_info(uuid)?;
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();
    if name.is_empty() {
        anyhow::bail!("插件信息缺少 name: {}", uuid);
    }
    if value.get("enabled").and_then(Value::as_i64) == Some(0) {
        anyhow::bail!("插件已禁用: {} ({})", uuid, name);
    }
    let install_path = value
        .get("install_path")
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string();

    Ok(PluginInfo {
        uuid: uuid.to_string(),
        name,
        install_path,
    })
}

/// 调用插件的 `post_chat`，原样传入请求并返回插件响应。
pub fn call_chat(info: &PluginInfo, request: &str) -> Result<String> {
    let library = plugin_library(info)?;
    let function = unsafe { library.get::<FfiFn>(PLUGIN_ENTRY) }
        .with_context(|| format!("插件 {} 缺少导出符号 post_chat", info.name))?;

    let request = CString::new(request).context("请求 JSON 含 \\0")?;
    let mut out = vec![0u8; OUT_BUF_SIZE];
    let ret = unsafe { function(request.as_ptr(), out.as_mut_ptr() as *mut c_char) };

    // 旧插件的业务错误码会作为 C 返回值；此时响应体仍是有效 JSON，需要原样转发。
    if ret < 0 {
        anyhow::bail!("插件 {} FFI 返回 {}", info.name, ret);
    }

    let text = unsafe { CStr::from_ptr(out.as_ptr() as *const c_char) }
        .to_str()
        .with_context(|| format!("插件 {} 响应非 UTF-8", info.name))?;
    if text.is_empty() {
        anyhow::bail!("插件 {} 返回空响应（ret={}）", info.name, ret);
    }
    Ok(text.to_string())
}

fn plugin_library(info: &PluginInfo) -> Result<Arc<Library>> {
    let cache = PLUGIN_LIBS.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(library) = cache.lock().unwrap().get(&info.name) {
        return Ok(library.clone());
    }

    let path = locate(info)?;
    let library = unsafe { Library::new(&path) }
        .with_context(|| format!("加载插件库 {} 失败", path.display()))?;
    let library = Arc::new(library);
    cache
        .lock()
        .unwrap()
        .insert(info.name.clone(), library.clone());
    Ok(library)
}

/// 按安装约定探测插件动态库。
fn locate(info: &PluginInfo) -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if !info.install_path.is_empty() {
        let dir = Path::new(&info.install_path);
        candidates.push(dir.join(format!("{}.{}", info.name, DYLIB_EXT)));
        candidates.push(dir.join(format!("lib{}.{}", info.name, DYLIB_EXT)));
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case(DYLIB_EXT))
                    .unwrap_or(false)
                {
                    candidates.push(path);
                }
            }
        }
    }
    candidates.push(PathBuf::from(format!(
        "./plugins/{0}/{0}.{1}",
        info.name, DYLIB_EXT
    )));
    candidates.push(PathBuf::from(format!(
        "./plugins/{0}/lib{0}.{1}",
        info.name, DYLIB_EXT
    )));
    candidates.push(PathBuf::from(format!(
        "./libary/{0}.{1}",
        info.name, DYLIB_EXT
    )));
    candidates.push(PathBuf::from(format!(
        "./libary/lib{0}.{1}",
        info.name, DYLIB_EXT
    )));

    candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("找不到插件 {} 的动态库", info.name))
}
