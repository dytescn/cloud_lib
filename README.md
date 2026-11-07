# cloud_lib

`cloud_lib` 是 Piksel 的统一云端大模型调用库。Rust `cdylib`，由宿主通过
`POST /cloud_lib` + `FFI-Symbol: chat` 调用，也可以由 chatllm 直接 `dlopen`
`./libary/cloud_lib.{dll,so,dylib}`。

它合并了原先 `deepseek` 与 `siliconflow_t2t` 两个系统库的调用实现，并接管
「按 `ai_supplier` 选择供应商」的分发逻辑，对外只暴露一个 FFI 入口 `post_chat`。

## 包信息

| 项 | 内容 |
| --- | --- |
| 仓库 | `dytescn/cloud_lib` |
| 包名 / 类型 | `cloud_lib` / `sys` |
| 版本 | `1.0.0` |
| 产物 | Windows `cloud_lib.dll`、Linux `cloud_lib.so`、macOS `cloud_lib.dylib` |
| 入口 | `post_chat` |
| 依赖 | `database`（读 `info.db.setting`）、`info_lib`（`setup.json` 写入配置） |
| 配置前缀 | `deepseek_app_*` / `silicon_app_*` |

## 调用方式

```c
int post_chat(const char *json_in, char *json_out);
```

- `json_in`：UTF-8 JSON；
- `json_out`：调用方提供 64 KiB 缓冲区；
- 返回 `0` 表示已写入；`-1` 输出指针为空；`-2` 缓冲区不足；
- 业务结果在响应 JSON 的 `code` 中。

请求：

```json
{
  "supplier": "deepseek",
  "prompt": "你好",
  "messages": [{ "role": "user", "content": "你好" }],
  "system": "你是一个中文助手",
  "model": "deepseek-flash",
  "max_tokens": 4096,
  "temperature": 0.7,
  "top_p": 0.7,
  "top_k": 50,
  "thinking": "enabled",
  "reasoning_effort": "high",
  "session_id": "biz-id"
}
```

- `supplier`：可选，`deepseek` / `siliconflow_t2t`；缺省读取
  `info.db.setting.ai_supplier`，未配置时回退 `deepseek`；
- `prompt` 与 `messages` 至少提供一个；`messages` 优先；
- 其余字段均为可选，未传时使用对应供应商的设置；
- `top_p` / `top_k` 只对 SiliconFlow 生效，`reasoning_effort` 只对 DeepSeek 生效；
- `session_id` 仅回显。

响应：

```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "content": "模型回复",
    "id": "msg_xxx",
    "model": "deepseek-flash",
    "stop_reason": "end_turn",
    "usage": {},
    "supplier": "deepseek",
    "session_id": "biz-id"
  }
}
```

## 配置

`setup.json` 通过 `info_lib` 写入以下 `setting`，均使用 `allow_error` 保证幂等：

| 供应商 | key | 默认值 |
| --- | --- | --- |
| DeepSeek | `deepseek_app_key` | 占位符，必须替换 |
| | `deepseek_app_base_url` | `https://api.deepseek.com/anthropic` |
| | `deepseek_app_model` | `deepseek-flash` |
| | `deepseek_app_max_tokens` | `4096` |
| | `deepseek_app_temperature` | `0.7` |
| | `deepseek_app_system_prompt` | 你是一个乐于助人的中文 AI 助手。 |
| | `deepseek_app_thinking` | `disabled` |
| | `deepseek_app_reasoning_effort` | `high` |
| SiliconFlow | `silicon_app_key` | 占位符，必须替换 |
| | `silicon_app_base_url` | `https://api.siliconflow.cn/v1` |
| | `silicon_app_model` | `Qwen/Qwen2.5-7B-Instruct` |
| | `silicon_app_max_tokens` | `4096` |
| | `silicon_app_temperature` | `0.7` |
| | `silicon_app_top_p` | `0.7` |
| | `silicon_app_topk` | `40`（源码缺省 `50`） |
| | `silicon_app_system_prompt` | 你是一个乐于助人的中文 AI 助手。 |

`ai_supplier` 由 setting 页面维护，不在本包初始化，避免覆盖用户选择。

## 错误码

| 码 | 含义 |
| --- | --- |
| `0` | 成功 |
| `400001` | `json_in` 为空 |
| `400002` | 请求 JSON 解析失败 |
| `400003` | 请求不是 UTF-8 |
| `400004` | 未知供应商 |
| `400102` | `prompt` / `messages` 没有有效内容 |
| `500101` | 读取配置、HTTP 调用或响应解析失败 |
| `500201` | 读取 `info.db.setting` 失败 |
| `500900` | `json_out` 为空 |
| `500901` | 响应超过 64 KiB |

## 构建与测试

```bash
cargo build --release
cargo test
```

## 迁移说明

原先 `chatllm` 内的 `cloud.rs` 会按 `ai_supplier` 直接 `dlopen`
`libdeepseek` / `libsiliconflow_t2t`。现在：

- 供应商实现统一收口到本库；
- `chatllm` 只加载 `cloud_lib`，不再感知具体供应商库；
- 原 `deepseek` / `siliconflow_t2t` 包已被本包取代，源码从 `dytescn/chat_plugins` 移除。

接口细节见 [docs/ffi.md](docs/ffi.md)。
