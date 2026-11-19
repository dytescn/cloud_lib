# 测试环境规范

本文件沿用 dytes 项目的测试环境约定，并适配到 cloud_lib。

## 1. 测试目录

- 所有本地测试的临时文件统一放在包目录的 `.dytes/` 下
  （如 `chat_t2t/.dytes/`、`deepseek/.dytes/`）。
- `.dytes/` 应写入包根 `.gitignore`；不得提交任何测试产物。
- 每个测试使用独立子目录：`.dytes/run-<pid>/<test-name>/`，避免并发冲突。
- 测试依赖统一下载到 `.dytes/env/`，不修改真实安装目录。
- 测试结束保留现场便于排查；整个 `.dytes/` 可随时手动删除。

## 2. 目录结构

```text
.dytes/
├── env/                 # 测试依赖环境（每次重建）
│   ├── dytes.json       # 合并后的测试依赖清单
│   ├── dytes.lock       # 依赖安装记录
│   ├── libary/
│   ├── web/<name>/
│   ├── plugins/<name>/
│   ├── apps/<name>/
│   └── models/<name>/
└── run-<pid>/
    └── <test-name>/
        ├── work/        # 命令工作目录
        ├── cache/       # 下载缓存
        ├── logs/        # 输出与日志
        └── config.toml  # DYTES_CONFIG
```

`dytes test` 的依赖来源由 `dytes.json` 的 `test-dependency` 与
`dependencies-*` 共同决定：

```json
"test-dependency": {
  "sys": { "info_lib": "0.1.0" },
  "web": { "setting": "0.2.0" }
}
```

当前包自身不下载，使用本地构建产物；依赖按声明 semver 从 S3 下载线上最新
版本到 `.dytes/env/`，失败即报错、不回退本地复制。

## 3. 环境隔离

- `DYTES_CONFIG` 指向 `.dytes/.../config.toml`，禁止读写用户真实配置目录
  （`%APPDATA%\dytes`、`~/.config/dytes`）。
- 清除 `DYTES_S3_*`、`PIKSEL_PLATFORM` 等外部环境变量，避免本机环境影响结果。
- 平台相关用例固定 `DYTES_PLATFORM=windows`，或按用例显式指定。
- 默认不访问网络；需要真实 S3 / 上游模型的测试必须显式标记并跳过。

## 4. 测试代码

- 测试统一放在 `tests/`，不写入 `src/`（见 [code.md](code.md) 3）。
- 共享辅助代码放在 `tests/common/mod.rs`。
- 纯逻辑测试直接 `cargo test`，不需要动态库。
- FFI 集成测试在 `.dytes/` 下执行，加载 `.dytes/libary/` 中的真实动态库；
  `cloud_lib` 通过相对当前工作目录的 `./libary/database.{dll,so,dylib}` 加载
  `database`，路径不对会返回 `500201`。
- 每次构建后把产物覆盖到 `.dytes/libary/`，确保测试加载的是最新代码。
- 依赖库文件名以 dytes 实际安装到 `libary/` 的名称为准。

## 5. 没有 S3 配置时

本地 `cargo build` 默认**不包含**内置 S3 凭据（只有构建时提供 `UPYUN_API_KEY` /
`UPYUN_API_SECRET` 才会注入）。因此 `dytes config list` 显示
`s3.access_key` / `s3.secret_key = (未设置)` 时：

- `install` / `update` / `publish` / `self-update` 访问对象存储会失败；
- `packages.funxdata.com` 上的清单由发布流程更新，并可能受 CDN 缓存影响，
  不能保证立即拿到最新包。

处理方式：

1. 配置凭据后再测试：

   ```bash
   dytes config set s3.access_key
   dytes config set s3.secret_key
   ```

   或使用环境变量 `DYTES_S3_ACCESS_KEY` / `DYTES_S3_SECRET_KEY`。

2. 先确认线上版本，再判断问题：

   ```bash
   curl -fsSL "https://packages.funxdata.com/core/dytes/dytes.json?t=$(date +%s)"
   ```

   版本落后时先完成发布，不要用旧包验证新功能。

3. 只测本地逻辑时，使用 `.dytes/` 隔离目录与 `DYTES_CONFIG`，不依赖线上包。

## 6. 发布 → Action → 校验 → 测试

需要基于最新发布版本测试时，按以下顺序执行：

1. 同步三处版本号（`dytes.json`、`Cargo.toml`、`Cargo.lock`），提交并打 tag
   （tag 前缀见 [workflows.md](workflows.md)）：

   ```bash
   git tag <目录名>-vX.Y.Z
   git push origin main --tags
   ```

2. 等待对应 workflow 完成，确认三平台产物和清单已更新。

3. 用当前 `dytes` 确认线上版本：

   ```bash
   curl -fsSL "https://packages.funxdata.com/<分类>/<包名>/dytes.json?t=$(date +%s)"
   dytes --version
   ```

4. 版本确认后再执行本地测试：

   ```bash
   cargo build --release
   cargo test
   ```

5. 如果拿到的仍是旧版本，优先检查 CDN 缓存（清单 URL 加时间戳）或发布是否
   成功，不要反复重装。

## 7. 后续补充

- 测试数据（fixtures）与清理策略；
- CI 中的 `.dytes` 缓存与保留期限；
- 需要凭据的集成测试标记方式。
