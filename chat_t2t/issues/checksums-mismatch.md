# [P1] `dytes.json` 的 `setup.json` checksum 与实际文件不一致

- 发现日期：2026-11-10
- 包：`cloud_lib`（`type: sys`）
- 线上版本：`1.0.0`
- 文件：`system/cloud_lib/1.0.0/windows/setup.json`

## 现象

使用 `dytes install` / `dytes update` 安装 Piksel 时：

```text
sys cloud_lib: 校验失败，已删除文件: .../libary/setup.json:
文件大小不匹配（期望 17526，实际 18171）
```

清单 `dytes.json` 记录：

```json
"checksums": {
  "setup.json": {
    "size": 17526,
    "sha256": "b98d644ec5eb6de74190a6b9d266165f64b3e0d8c1cf35131f43a8b85ac0586a"
  }
}
```

实际下载的文件：

```text
size:   18171
sha256: d8da0ad07f43a37d677afba28829f7ea6b8a7ce02a982b99040cb30c3e3fd75c
```

## 影响

- `dytes` 的 `verify_download` 在大小或哈希不匹配时删除文件并中止该包更新；
- `cloud_lib` 无法安装，`dytes install` 最终返回非 0；
- 依赖 `cloud_lib` 的功能不可用。

## 修复建议

1. 基于最终上传的 `setup.json` 重新生成 `checksums.size` / `checksums.sha256`；
2. 发布新的 patch 版本（例如 `1.0.1`），不要覆盖 `1.0.0` 的版本化路径；
3. 刷新 `packages.funxdata.com` 对应 CDN 缓存；
4. 使用 `dytes update --type sys` 验证不再出现大小/哈希不匹配。

## 复现

```bash
curl -fsSL "https://packages.funxdata.com/system/cloud_lib/1.0.0/dytes.json" | grep -A8 checksums
curl -fsSL "https://packages.funxdata.com/system/cloud_lib/1.0.0/windows/setup.json" | wc -c
```
