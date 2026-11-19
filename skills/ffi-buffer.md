# 库间 FFI 缓冲区约定

本文件沿用 designplugin_lib 的缓冲区约定，并记录 cloud_lib 的实际情况。

## 1. 适用范围

| 调用方向 | 示例 | 约定 |
| --- | --- | --- |
| 对外 FFI | `cyctron` / `chatllm` → `cloud_lib` | **64 KiB** |
| 库与库之间 | `cloud_lib` → `database` | **1024 KiB**，可按依赖约定扩大 |

两类缓冲区**不得共用同一个常量**。修改库间缓冲区属于 FFI 契约调整，
按 [dev.md](./dev.md) 的规则先由人工确认。

## 2. 当前实现

| 调用 | 常量位置 | 当前值 | 说明 |
| --- | --- | --- | --- |
| 对外 FFI | `src/utils/json.rs` `OUT_BUF_SIZE` | 64 KiB | 调用方分配；不足时返回 `-2` |
| `cloud_lib` → `database` | `src/utils/db.rs` `OUT_BUF_SIZE` | 1024 KiB | database 1.0.5+ 约定至少 1 MiB |

## 3. 修改约定

1. 缓冲区变更先确认对端库的最大响应，再调整常量；
2. 对外 FFI 的 64 KiB 约定不要被库间缓冲区误改；
3. 变更时同步更新本节表格与 `README.md`；
4. 增加边界测试：接近上限、超过上限、对端返回超大响应的场景。
