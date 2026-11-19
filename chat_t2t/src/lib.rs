//! cloud —— 统一的大模型云端调用库。
//!
//! 把原先分散在 `deepseek` / `siliconflow_t2t` 两个系统库中的供应商调用合并到一个
//! `cdylib`，对外只暴露一个 FFI 入口 [`ffi::chat_t2t`]。调用方（chatllm）不再
//! `dlopen` 具体供应商库，从而避免插件回调与 chatllm 互相加载的问题。
//!
//! 模块划分：
//! - [`ffi`]：唯一的 C ABI 入口；
//! - [`utils`]：配置读取、JSON 辅助、供应商定义与具体实现。

pub mod ffi;
pub mod utils;
