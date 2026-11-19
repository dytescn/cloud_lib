# cloud_lib

`cloud_lib` 是 Piksel 的统一云端大模型调用库。Rust `cdylib`，由宿主通过
`POST /cloud_lib` + `FFI-Symbol: chat_t2t` 调用，也可以由 chatllm 直接 `dlopen`
`./libary/cloud_lib.{dll,so,dylib}`。

它接管「按 `ai_supplier` 选择插件」的职责，对外只暴露一个 FFI 入口 `chat_t2t`：
读取 `setting.ai_supplier`（插件 uuid）→ 通过 `plugin_lib` 读取插件信息 →
加载插件动态库并调用其 `post_chat`，把插件响应原样返回。

## 包信息

| 项 | 内容 |
| --- | --- |
| 仓库 | `dytescn/cloud_lib` |
| 包名 / 类型 | `cloud_lib` / `sys` |
| 版本 | `1.0.0` |
| 产物 | Windows `cloud_lib.dll`、Linux `cloud_lib.so`、macOS `cloud_lib.dylib` |
| 入口 | `chat_t2t` |
| 依赖 | `database`（读 `info.db.setting`）、`plugin_lib`（查插件信息）、`info_lib`（`setup.json` 写入配置） |
| 配置项 | `ai_supplier`（插件 uuid） / `ai_supplier_plugin_uuid`（注册参数） |

## 调用方式

```c
int chat_t2t(const char *json_in, char *json_out);
```

- `json_in`：UTF-8 JSON，**原样转发**给插件；
- `json_out`：调用方提供 1 MiB 缓冲区；
- 返回 `0` 表示已写入；`-1` 输出指针为空；`-2` 缓冲区不足；
- 业务结果以插件响应为准，插件应返回 `{ code, msg, data }` 结构。

请求示例（字段由插件约定）：

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

请求字段由插件定义；`chat_t2t` 不做校验与改写，只负责选择插件并转发。
示例响应（由插件返回）：

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

| key | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `ai_supplier` | `enum` | `c39f6e4d-042a-4b3e-bc8d-2a7c3336736c` | 当前插件的 uuid；切换选项即切换插件 |
| `ai_supplier_plugin_uuid` | `string` | 空 | 供其它方法注册使用，切换选项时写入对应插件 uuid |

`ai_supplier` 的可选值就是插件 uuid：

| 插件 | uuid |
| --- | --- |
| `deepseek` | `c39f6e4d-042a-4b3e-bc8d-2a7c3336736c` |
| `siliconflow_t2t` | `c8b1acac-2b36-463b-84f7-c2170d31ab63` |

`chat_t2t` 只读取 `ai_supplier`：未设置（或为空）时返回 `500202`，不做默认回退。
各插件自己的配置（API Key、模型等）由插件包各自的 `setup.json` 维护。

## 错误码

| 码 | 含义 |
| --- | --- |
| `0` | 成功（响应由插件返回，实际以插件 `code` 为准） |
| `400001` | `json_in` 为空 |
| `400003` | 请求不是 UTF-8 |
| `500101` | 读取插件信息或调用插件失败 |
| `500201` | 读取 `info.db.setting` 失败 |
| `500202` | 未设置 `ai_supplier`（插件 uuid） |

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
- 原 `deepseek` / `siliconflow_t2t` 包从 `dytescn/chat_plugins` 移出，源码归档在
  本仓库根目录的 `deepseek/` 与 `siliconflow_t2t/` 目录（即本目录 `chat_t2t/` 的同级目录）。

## 历史实现

`../deepseek/` 与 `../siliconflow_t2t/` 是迁移前的独立包源码，随本仓库保留用于对照与追溯；
它们不参与本目录 `cloud_lib` 的构建，也不进入 `dytes.json` 的发布产物。
当前 `src/utils/` 只保留配置读取、错误码、JSON 辅助、供应商定义与插件查询/调用等
通用模块，不再包含供应商调用实现。

接口细节见 [docs/ffi.md](docs/ffi.md)。
