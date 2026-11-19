# Workflows 打包与发布规范

本仓库按「一个包目录一个 workflow」组织发布流程。当前目录与文件一一对应：

| 包目录 | workflow | tag 前缀 |
| --- | --- | --- |
| `chat_t2t/` | `.github/workflows/chat_t2t.yml` | `chat_t2t-v*` |
| `deepseek/` | `.github/workflows/deepseek.yml` | `deepseek-v*` |
| `siliconflow_t2t/` | `.github/workflows/siliconflow_t2t.yml` | `siliconflow_t2t-v*` |

## 1. 文件命名

- workflow 文件名必须与包目录同名：`<目录名>.yml`。
- 一个 workflow 只负责一个包目录，不得在一个 workflow 中混合构建多个包。
- 包目录新增 / 重命名 / 删除时，同步新增 / 重命名 / 删除对应 workflow。

## 2. 触发规则

- 每个包使用独立的 tag 前缀 `<目录名>-v<版本>`，例如 `chat_t2t-v1.0.4`。
- 不得使用共用的 `v*` 前缀，避免一个 tag 触发全部包。
- 保留 `workflow_dispatch`，便于手动发布。
- `push.tags` 只写本包前缀，不要写其他包的前缀。

## 3. 工作目录

- job 级统一声明：

  ```yaml
  defaults:
    run:
      working-directory: <目录名>
  ```

- 所有 `run` 步骤（读取 `dytes.json`、`cargo build --release`、`cmake`）都在
  包目录内执行。
- `uses: dytescn/dytes` 步骤不受 `defaults.run` 影响，必须显式传 `dir`：

  ```yaml
  with:
    command: publish
    dir: <目录名>
  ```

- 不得在仓库根目录执行 `cargo build` 或读取 `dytes.json`。

## 4. 版本校验

- 发布前校验 tag 与包内 `dytes.json.version` 一致，例如：

  ```bash
  tag_version="${GITHUB_REF_NAME#<目录名>-v}"
  if [ "${tag_version}" = "${GITHUB_REF_NAME}" ] || [ "${tag_version}" != "${version}" ]; then
    echo "tag 与 dytes.json 版本不一致" >&2
    exit 1
  fi
  ```

- 打 tag 前同步三处版本号：`dytes.json`、`Cargo.toml`、`Cargo.lock`。

## 5. 构建方式

由 workflow 自动探测，不在配置中写死：

- 存在 `CMakeLists.txt` → `cmake`；
- 否则存在 `Cargo.toml` → `cargo`；
- 否则 → `deno`。

平台列表从 `dytes.json` 的 `platform-exports` 读取，逐平台并行发布
（`max-parallel: 1`）。

## 6. 发布产物

- 产物路径以包内 `dytes.json` 的 `platform-exports` 为准
  （如 `./target/release/cloud_lib.dll`），均相对包目录。
- 不得在 workflow 中手写产物路径或复制文件到其他目录。
- `setup.json` 随包发布，`init` 列表保持不变。

## 7. 新增包的步骤

1. 在仓库根目录创建包目录，放入 `dytes.json`、`Cargo.toml`、`setup.json` 等。
2. 复制任一现有 workflow，重命名为 `<目录名>.yml`。
3. 替换所有 `<目录名>`：`name`、`tags`、`working-directory`、tag 前缀解析、`dir`。
4. 本地验证：

   ```bash
   cd <目录名>
   cargo build --release
   dytes check
   ```

5. 提交后使用 `<目录名>-v<版本>` 打 tag 发布。

## 8. 禁止事项

- 不得在根目录保留 `dytes-release.yml` 一类跨包 workflow。
- 不得让 workflow 引用其他包目录的文件或产物。
- 不得在发布流程中修改源码、`dytes.json` 或 `Cargo.toml` 版本。
