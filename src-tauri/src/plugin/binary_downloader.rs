//! 二进制组件下载器
//!
//! 从 GitHub Releases 下载二进制组件

use reqwest::Client;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, warn};

/// GitHub Release Asset 信息
#[derive(Debug, Clone)]
pub struct ReleaseAsset {
    /// 文件名
    pub name: String,
    /// 下载 URL
    pub download_url: String,
    /// 文件大小 (bytes)
    pub size: u64,
}

/// 二进制组件下载器
pub struct BinaryDownloader {
    client: Client,
}

impl BinaryDownloader {
    /// 创建新的下载器
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .user_agent("ProxyCast")
                .build()
                .expect("Failed to create HTTP client"),
        }
    }

    /// 获取当前平台标识
    pub fn get_platform_key() -> &'static str {
        match (std::env::consts::ARCH, std::env::consts::OS) {
            ("aarch64", "macos") => "macos-arm64",
            ("x86_64", "macos") => "macos-x64",
            ("x86_64", "linux") => "linux-x64",
            ("aarch64", "linux") => "linux-arm64",
            ("x86_64", "windows") => "windows-x64",
            _ => "unknown",
        }
    }

    /// 获取当前平台的二进制文件名
    pub fn get_platform_binary_name(base_name: &str) -> String {
        match (std::env::consts::ARCH, std::env::consts::OS) {
            ("aarch64", "macos") => format!("{base_name}-aarch64-apple-darwin"),
            ("x86_64", "macos") => format!("{base_name}-x86_64-apple-darwin"),
            ("x86_64", "linux") => format!("{base_name}-x86_64-unknown-linux-gnu"),
            ("aarch64", "linux") => format!("{base_name}-aarch64-unknown-linux-gnu"),
            ("x86_64", "windows") => format!("{base_name}-x86_64-pc-windows-msvc.exe"),
            _ => format!("{base_name}-unknown"),
        }
    }

    /// 获取最新版本信息
    pub async fn get_latest_version(
        &self,
        github_owner: &str,
        github_repo: &str,
    ) -> Result<(String, Vec<ReleaseAsset>), String> {
        let api_url =
            format!("https://api.github.com/repos/{github_owner}/{github_repo}/releases/latest");

        info!("获取最新版本: {}", api_url);

        let response = self
            .client
            .get(&api_url)
            .header("Accept", "application/vnd.github.v3+json")
            .send()
            .await
            .map_err(|e| format!("请求 GitHub API 失败: {e}"))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("GitHub API 请求失败: {status} - {body}"));
        }

        let data: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("解析响应失败: {e}"))?;

        let version = data["tag_name"]
            .as_str()
            .unwrap_or("")
            .trim_start_matches('v')
            .to_string();

        let assets = data["assets"]
            .as_array()
            .ok_or("未找到 assets")?
            .iter()
            .filter_map(|a| {
                Some(ReleaseAsset {
                    name: a["name"].as_str()?.to_string(),
                    download_url: a["browser_download_url"].as_str()?.to_string(),
                    size: a["size"].as_u64().unwrap_or(0),
                })
            })
            .collect();

        info!(
            "最新版本: {}, 找到 {} 个 assets",
            version,
            data["assets"].as_array().map(|a| a.len()).unwrap_or(0)
        );

        Ok((version, assets))
    }

    /// 下载二进制文件（带进度回调）
    pub async fn download_binary<F>(
        &self,
        download_url: &str,
        target_path: &PathBuf,
        progress_callback: F,
    ) -> Result<(), String>
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        info!("开始下载: {} -> {:?}", download_url, target_path);

        let response = self
            .client
            .get(download_url)
            .send()
            .await
            .map_err(|e| format!("下载请求失败: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("下载失败: HTTP {}", response.status()));
        }

        let total_size = response.content_length().unwrap_or(0);
        info!("文件大小: {} bytes", total_size);

        // 确保目标目录存在
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| format!("创建目录失败: {e}"))?;
        }

        let mut file = fs::File::create(target_path)
            .await
            .map_err(|e| format!("创建文件失败: {e}"))?;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();

        use futures::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("读取数据失败: {e}"))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("写入文件失败: {e}"))?;
            downloaded += chunk.len() as u64;
            progress_callback(downloaded, total_size);
        }

        file.flush()
            .await
            .map_err(|e| format!("刷新文件失败: {e}"))?;

        // 设置可执行权限 (Unix)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let metadata = fs::metadata(target_path)
                .await
                .map_err(|e| format!("获取文件权限失败: {e}"))?;
            let mut perms = metadata.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(target_path, perms)
                .await
                .map_err(|e| format!("设置可执行权限失败: {e}"))?;
        }

        info!("下载完成: {:?}", target_path);
        Ok(())
    }

    /// 验证校验和
    pub async fn verify_checksum(
        &self,
        file_path: &PathBuf,
        expected_hash: &str,
    ) -> Result<bool, String> {
        let content = fs::read(file_path)
            .await
            .map_err(|e| format!("读取文件失败: {e}"))?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = format!("{:x}", hasher.finalize());

        let matches = result.to_lowercase() == expected_hash.to_lowercase();
        if !matches {
            warn!("校验和不匹配: 期望 {}, 实际 {}", expected_hash, result);
        }

        Ok(matches)
    }

    /// 下载并解析 checksums.txt
    pub async fn get_checksums(
        &self,
        assets: &[ReleaseAsset],
        checksum_filename: &str,
    ) -> Result<HashMap<String, String>, String> {
        let checksum_asset = assets
            .iter()
            .find(|a| a.name == checksum_filename)
            .ok_or_else(|| format!("未找到校验文件: {checksum_filename}"))?;

        let response = self
            .client
            .get(&checksum_asset.download_url)
            .send()
            .await
            .map_err(|e| format!("下载校验文件失败: {e}"))?;

        if !response.status().is_success() {
            return Err(format!("下载校验文件失败: HTTP {}", response.status()));
        }

        let content = response
            .text()
            .await
            .map_err(|e| format!("读取校验文件失败: {e}"))?;

        let mut checksums = HashMap::new();
        for line in content.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                // 格式: hash  filename 或 hash *filename
                let hash = parts[0];
                let filename = parts[1].trim_start_matches('*');
                checksums.insert(filename.to_string(), hash.to_string());
            }
        }

        info!("解析到 {} 个校验和", checksums.len());
        Ok(checksums)
    }

    /// 获取插件目录
    pub fn get_plugins_dir() -> Result<PathBuf, String> {
        dirs::config_dir()
            .ok_or_else(|| "无法获取配置目录".to_string())
            .map(|p| p.join("proxycast").join("plugins"))
    }

    /// 获取特定组件的目录
    pub fn get_component_dir(component_name: &str) -> Result<PathBuf, String> {
        Self::get_plugins_dir().map(|p| p.join(component_name))
    }
}

impl Default for BinaryDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_key() {
        let key = BinaryDownloader::get_platform_key();
        assert!(!key.is_empty());
        // 在测试环境中应该返回有效的平台标识
        assert!([
            "macos-arm64",
            "macos-x64",
            "linux-x64",
            "linux-arm64",
            "windows-x64",
            "unknown"
        ]
        .contains(&key));
    }

    #[test]
    fn test_get_platform_binary_name() {
        let name = BinaryDownloader::get_platform_binary_name("aster-server");
        assert!(name.starts_with("aster-server-"));
    }

    #[test]
    fn test_get_plugins_dir() {
        let result = BinaryDownloader::get_plugins_dir();
        assert!(result.is_ok());
        let path = result.unwrap();
        assert!(path.ends_with("proxycast/plugins") || path.ends_with("proxycast\\plugins"));
    }
}
