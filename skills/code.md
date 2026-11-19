# 代码规范

本文件记录 cloud_lib 的目录与代码组织规范。后续按需逐步增加条目。

## 1. 目录组织

- `src/` 主目录只保留 `lib.rs`，其余代码按职责放入 `src/ffi/` 与 `src/utils/`。
- `src/ffi/`：唯一的 C ABI 入口（`chat_t2t`）及其请求/响应结构。
- `src/utils/`：配置读取、JSON 辅助与供应商定义。
- 新增模块放入 `utils/`（或明确的子目录），不在 `src/` 根下新增文件。

```text
src/
├── lib.rs
├── ffi/
│   ├── mod.rs
│   └── chat.rs
└── utils/
    ├── mod.rs
    ├── db.rs
    ├── errcode.rs
    ├── json.rs
    ├── plugin.rs
    ├── setting.rs
    └── supplier.rs
```

## 2. 不在 `mod.rs` 中编写代码

`mod.rs` 只允许声明子模块（`pub mod`）和必要的重导出（`pub use`），
不允许写逻辑代码、常量、测试等；模块入口只负责组装。

正确：

```rust
pub mod ffi;
pub mod utils;
```

```rust
pub mod db;
pub mod errcode;
pub mod json;
pub mod plugin;
pub mod setting;
pub mod supplier;
```

错误：

```rust
pub mod db;

pub fn load() { /* 逻辑代码 */ }

pub const OUT_BUF_SIZE: usize = 1024 * 1024;
```

需要对外暴露的条目用 `pub use` 重导出，实现仍放在独立文件中。

## 3. 测试不写在 `src/` 的源码文件中

`src/` 下的源码文件不得包含 `#[cfg(test)] mod tests` 等测试代码；
测试统一放在仓库根目录的 `tests/` 目录中，按模块拆分为独立测试文件。

```text
tests/
├── json_test.rs
└── supplier_test.rs
```

测试应通过公共 API 验证行为；需要测试内部实现时，优先重构为可测试的
公共接口，而不是把测试塞回 `src/`。

> 存量说明：当前 `src/utils/` 内仍有内联测试，后续按模块逐步移入 `tests/`；
> 新增测试必须遵守本条。
