//! 插件安装器模块
//!
//! 提供插件安装、卸载和管理功能：
//! - 从本地文件安装插件
//! - 从 URL（如 GitHub releases）下载安装插件
//! - 插件包验证
//! - 插件注册表管理
//! - 安装进度回调

mod downloader;
mod installer;
mod registry;
mod types;
mod validator;

pub use downloader::PluginDownloader;
pub use installer::PluginInstaller;
pub use registry::PluginRegistry;
pub use types::{
    GitHubRelease, InstallError, InstallProgress, InstallSource, InstallStage, InstalledPlugin,
    NoopProgressCallback, PackageFormat, ProgressCallback,
};
pub use validator::PackageValidator;
