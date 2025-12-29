# ProxyCast 插件目录

此目录包含 ProxyCast 的官方插件源代码。

## 目录结构

```
plugins/
├── README.md                    # 本文件
├── machine-id-tool/             # Machine ID 管理工具插件
│   └── plugin.json              # 插件清单文件
└── ...                          # 其他插件
```

## 插件清单格式 (plugin.json)

每个插件必须包含一个 `plugin.json` 清单文件，定义插件的元数据：

```json
{
  "name": "plugin-name",
  "version": "0.1.0",
  "description": "插件描述",
  "author": "作者名",
  "homepage": "https://github.com/...",
  "license": "MIT",
  "plugin_type": "binary",
  "entry": "plugin-name",
  "hooks": [],
  "min_proxycast_version": "1.0.0",
  "binary": {
    "binary_name": "plugin-name",
    "github_owner": "owner",
    "github_repo": "repo",
    "platform_binaries": {
      "macos-arm64": "binary-aarch64-apple-darwin",
      "macos-x64": "binary-x86_64-apple-darwin",
      "linux-x64": "binary-x86_64-unknown-linux-gnu",
      "linux-arm64": "binary-aarch64-unknown-linux-gnu",
      "windows-x64": "binary-x86_64-pc-windows-msvc.exe"
    }
  },
  "ui": {
    "surfaces": ["main"],
    "icon": "icon-name",
    "title": "插件标题"
  }
}
```

## 构建插件包

使用构建脚本将插件打包为 zip 格式：

```bash
# 打包指定插件
./scripts/build-plugin.sh machine-id-tool

# 指定版本号
./scripts/build-plugin.sh machine-id-tool 0.2.0
```

输出文件将保存在 `dist/plugins/` 目录下。

## 插件类型

- `script`: 脚本插件（JSON 配置驱动）
- `native`: 原生 Rust 插件（预留）
- `binary`: 二进制可执行文件插件

## 相关文档

- [插件安装机制设计文档](.kiro/specs/plugin-installation/design.md)
- [插件系统 README](src-tauri/src/plugin/README.md)
