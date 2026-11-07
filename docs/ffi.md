# cloud_lib FFI

`cloud_lib` 编译为 `cdylib`，由宿主通过 `POST /cloud_lib` + `FFI-Symbol: chat`
调用（cyctron 会查找 `libary/` 下对应动态库并拼接 `post_` 前缀），
或者由 chatllm 直接 `dlopen`。运行时 cwd 为安装根目录，`./libary`、
`./database` 均相对该目录解析。

## 1. 调用约定

```c
int (*)(const char *json_in, char *json_out);
```

| 约定 | 说明 |
| --- | --- |
| 输入 | NUL 结尾的 UTF-8 JSON |
| 输出 | 调用方提供 65,536 字节缓冲区（含结尾 NUL）；超限返回 `-2` |
| 返回值 | `0` 已写入；`-1` 输出指针为空；`-2` 缓冲区不足 |
| 响应 | `{"code":0,"msg":"success","data":{...}}`；失败 `{"code":<错误码>,"msg":"<原因>","data":{}}` |

## 2. 请求

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `supplier` | string | 否 | `deepseek` / `siliconflow_t2t`；缺省读 `ai_supplier` |
| `prompt` | string | 二选一 | 单轮输入 |
| `messages` | array | 二选一 | `{role, content}`；非空时优先于 `prompt` |
| `system` | string | 否 | 请求级 system；缺省读 `<supplier>_app_system_prompt` |
| `model` | string | 否 | 覆盖设置 |
| `max_tokens` | number | 否 | 覆盖设置 |
| `temperature` | number | 否 | 覆盖设置 |
| `top_p` | number | 否 | 仅 SiliconFlow |
| `top_k` | number | 否 | 仅 SiliconFlow |
| `thinking` | string \| object | 否 | `"enabled"` / `"disabled"` 或 `{"type":"enabled"}` |
| `reasoning_effort` | string | 否 | 仅 DeepSeek，`high` / `max` |
| `session_id` | string | 否 | 仅回显 |

规则：

1. `messages` 非空时使用 `messages`，忽略 `prompt`；
2. `messages` 为空时回落 `prompt`，自动构造一条 user 消息；
3. 两者都没有有效内容返回 `400102`；
4. `system` 优先级：请求 `system` > 设置；首条已是 system 时不重复插入；
5. 未显式传 `supplier` 时读取 `info.db.setting.ai_supplier`，空值 / 未知值回退
   `deepseek`。

## 3. 响应

```json
{
  "code": 0,
  "msg": "success",
  "data": {
    "content": "模型回复",
    "id": "供应商响应 id",
    "model": "实际模型",
    "stop_reason": "end_turn",
    "usage": {},
    "supplier": "deepseek",
    "session_id": "请求回显"
  }
}
```

供应商未返回的字段为 `null`。

## 4. 供应商差异

| 项 | DeepSeek | SiliconFlow |
| --- | --- | --- |
| 端点 | `{base}/v1/messages` | `{base}/messages` |
| 鉴权 | `x-api-key` + `anthropic-version: 2023-06-01` | `Authorization: Bearer` |
| 专属参数 | `thinking`、`output_config.effort` | `top_p`、`top_k`、`thinking` |
| 默认模型 | `deepseek-flash` | `Qwen/Qwen2.5-7B-Instruct` |

两者都按 Anthropic Messages 兼容格式解析 `content[].text`，当前为同步阻塞
HTTP（`ureq`，120 秒超时）。
