# 插件安装器模块

提供插件安装、卸载和管理功能。

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块入口，导出公共 API |
| `types.rs` | 类型定义：InstallError、InstallProgress、InstallStage、InstalledPlugin 等 |
| `validator.rs` | 包验证器：验证插件包格式（zip/tar.gz）和清单文件（plugin.json） |
| `downloader.rs` | 下载器：从 URL 下载插件包，支持 GitHub releases |
| `registry.rs` | 注册表：管理已安装插件的元数据（SQLite） |
| `installer.rs` | 安装器核心：协调整个安装/卸载流程 |

## Tauri 命令

插件安装功能通过以下 Tauri 命令暴露给前端（定义在 `commands/plugin_install_cmd.rs`）：

| 命令 | 说明 |
|------|------|
| `install_plugin_from_file` | 从本地文件安装插件 |
| `install_plugin_from_url` | 从 URL 安装插件（支持 GitHub releases） |
| `uninstall_plugin` | 卸载已安装的插件 |
| `list_installed_plugins` | 列出所有已安装插件 |
| `get_installed_plugin` | 获取指定插件的详细信息 |
| `is_plugin_installed` | 检查插件是否已安装 |

### 进度事件

安装过程中会通过 Tauri 事件 `plugin-install-progress` 发送进度更新，前端可以监听此事件显示安装进度。

## 核心类型

### InstallError
安装相关错误类型，包括下载失败、包格式无效、清单无效、校验和不匹配等。

### InstallProgress / InstallStage
安装进度和阶段，用于向前端报告安装状态。

### InstalledPlugin
已安装插件的元数据，包括 ID、名称、版本、安装路径等。

### InstallSource
安装来源：本地文件、URL、GitHub release。

### PackageFormat
包格式：Zip 或 TarGz。

## PackageValidator

包验证器提供以下功能：

- `validate_format(path)` - 验证包格式（zip/tar.gz），检测魔数和压缩包完整性
- `validate_manifest(manifest)` - 验证清单必需字段（name、version、entry、hooks）
- `validate_integrity(path, checksum)` - 验证 SHA256 校验和
- `extract_and_validate_manifest(path, format)` - 从压缩包提取并验证 plugin.json

### 验证规则

- 名称：只允许字母、数字、连字符、下划线，最长 64 字符
- 版本：semver 格式（x.y 或 x.y.z，可带后缀如 -beta）
- 钩子名称：只允许字母、数字、下划线、冒号

## 使用示例

```rust
use crate::plugin::installer::{
    PackageValidator, PluginDownloader, PluginRegistry,
    InstallProgress, NoopProgressCallback, PackageFormat,
};

// 验证包格式
let validator = PackageValidator::new();
let format = validator.validate_format(path)?;

// 提取并验证清单
let manifest = validator.extract_and_validate_manifest(path, format)?;

// 验证校验和（可选）
validator.validate_integrity(path, Some("sha256hash..."))?;

// 下载插件
let downloader = PluginDownloader::new();
downloader.download(url, dest, &NoopProgressCallback).await?;

// 注册插件
let registry = PluginRegistry::from_path(db_path)?;
registry.register(&plugin)?;
```
