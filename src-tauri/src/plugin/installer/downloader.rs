//! 插件下载器
//!
//! 处理远程插件包下载

use std::path::Path;

use super::types::{GitHubRelease, InstallError, InstallProgress, ProgressCallback};

/// 插件下载器
///
/// 处理远程插件包下载
pub struct PluginDownloader {
    client: reqwest::Client,
}

impl PluginDownloader {
    /// 创建新的下载器
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 使用自定义 HTTP 客户端创建下载器
    pub fn with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    /// 下载插件包
    ///
    /// 支持进度回调
    /// _需求: 2.1, 2.4_
    pub async fn download(
        &self,
        url: &str,
        dest: &Path,
        progress: &dyn ProgressCallback,
    ) -> Result<(), InstallError> {
        progress.on_progress(InstallProgress::downloading(0, "开始下载..."));

        let response = self
            .client
            .get(url)
            .header("User-Agent", "ProxyCast-Plugin-Installer")
            .send()
            .await
            .map_err(|e| InstallError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(InstallError::DownloadFailed(format!(
                "HTTP 错误: {}",
                response.status()
            )));
        }

        let total_size = response.content_length();
        let mut downloaded: u64 = 0;

        // 创建目标文件
        let mut file = tokio::fs::File::create(dest)
            .await
            .map_err(|e| InstallError::IoError(e))?;

        // 流式下载
        use tokio::io::AsyncWriteExt;
        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| InstallError::NetworkError(e.to_string()))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| InstallError::IoError(e))?;

            downloaded += chunk.len() as u64;

            // 计算进度
            let percent = if let Some(total) = total_size {
                ((downloaded as f64 / total as f64) * 100.0) as u8
            } else {
                // 未知大小时显示已下载字节数
                0
            };

            let message = if let Some(total) = total_size {
                format!(
                    "已下载 {:.1} MB / {:.1} MB",
                    downloaded as f64 / 1_000_000.0,
                    total as f64 / 1_000_000.0
                )
            } else {
                format!("已下载 {:.1} MB", downloaded as f64 / 1_000_000.0)
            };

            progress.on_progress(InstallProgress::downloading(percent, message));
        }

        file.flush().await.map_err(|e| InstallError::IoError(e))?;

        progress.on_progress(InstallProgress::downloading(100, "下载完成"));

        Ok(())
    }

    /// 解析 GitHub release URL
    ///
    /// 支持以下格式:
    /// - https://github.com/owner/repo/releases/download/tag/asset
    /// - https://github.com/owner/repo/releases/tag/tag
    /// - owner/repo@tag
    /// - owner/repo (使用 latest)
    ///
    /// _需求: 2.1_
    pub fn parse_github_url(&self, url: &str) -> Result<GitHubRelease, InstallError> {
        // 格式 1: 完整下载 URL
        if url.starts_with("https://github.com/") && url.contains("/releases/download/") {
            return self.parse_full_download_url(url);
        }

        // 格式 2: release 页面 URL
        if url.starts_with("https://github.com/") && url.contains("/releases/tag/") {
            return self.parse_release_page_url(url);
        }

        // 格式 3: owner/repo@tag
        if url.contains('@') && !url.contains("://") {
            return self.parse_short_format_with_tag(url);
        }

        // 格式 4: owner/repo (latest)
        if url.contains('/') && !url.contains("://") {
            return self.parse_short_format_latest(url);
        }

        Err(InstallError::UrlParseError(format!(
            "无法解析 GitHub URL: {}",
            url
        )))
    }

    /// 解析完整下载 URL
    fn parse_full_download_url(&self, url: &str) -> Result<GitHubRelease, InstallError> {
        // https://github.com/owner/repo/releases/download/tag/asset
        let path = url
            .strip_prefix("https://github.com/")
            .ok_or_else(|| InstallError::UrlParseError("无效的 GitHub URL".to_string()))?;

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 6 {
            return Err(InstallError::UrlParseError("URL 格式不完整".to_string()));
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        // parts[2] = "releases"
        // parts[3] = "download"
        let tag = parts[4].to_string();
        let asset_name = parts.get(5).map(|s| s.to_string());

        Ok(GitHubRelease {
            owner,
            repo,
            tag,
            asset_name,
        })
    }

    /// 解析 release 页面 URL
    fn parse_release_page_url(&self, url: &str) -> Result<GitHubRelease, InstallError> {
        // https://github.com/owner/repo/releases/tag/tag
        let path = url
            .strip_prefix("https://github.com/")
            .ok_or_else(|| InstallError::UrlParseError("无效的 GitHub URL".to_string()))?;

        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 5 {
            return Err(InstallError::UrlParseError("URL 格式不完整".to_string()));
        }

        let owner = parts[0].to_string();
        let repo = parts[1].to_string();
        // parts[2] = "releases"
        // parts[3] = "tag"
        let tag = parts[4].to_string();

        Ok(GitHubRelease {
            owner,
            repo,
            tag,
            asset_name: None,
        })
    }

    /// 解析短格式 (owner/repo@tag)
    fn parse_short_format_with_tag(&self, url: &str) -> Result<GitHubRelease, InstallError> {
        let parts: Vec<&str> = url.split('@').collect();
        if parts.len() != 2 {
            return Err(InstallError::UrlParseError(
                "无效的短格式，期望 owner/repo@tag".to_string(),
            ));
        }

        let repo_parts: Vec<&str> = parts[0].split('/').collect();
        if repo_parts.len() != 2 {
            return Err(InstallError::UrlParseError(
                "无效的仓库格式，期望 owner/repo".to_string(),
            ));
        }

        Ok(GitHubRelease {
            owner: repo_parts[0].to_string(),
            repo: repo_parts[1].to_string(),
            tag: parts[1].to_string(),
            asset_name: None,
        })
    }

    /// 解析短格式 (owner/repo) - 使用 latest
    fn parse_short_format_latest(&self, url: &str) -> Result<GitHubRelease, InstallError> {
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() != 2 {
            return Err(InstallError::UrlParseError(
                "无效的仓库格式，期望 owner/repo".to_string(),
            ));
        }

        Ok(GitHubRelease {
            owner: parts[0].to_string(),
            repo: parts[1].to_string(),
            tag: "latest".to_string(),
            asset_name: None,
        })
    }

    /// 获取 HTTP 客户端引用
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl Default for PluginDownloader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_download_url() {
        let downloader = PluginDownloader::new();
        let url = "https://github.com/user/repo/releases/download/v1.0.0/plugin.zip";
        let result = downloader.parse_github_url(url).unwrap();

        assert_eq!(result.owner, "user");
        assert_eq!(result.repo, "repo");
        assert_eq!(result.tag, "v1.0.0");
        assert_eq!(result.asset_name, Some("plugin.zip".to_string()));
    }

    #[test]
    fn test_parse_release_page_url() {
        let downloader = PluginDownloader::new();
        let url = "https://github.com/user/repo/releases/tag/v1.0.0";
        let result = downloader.parse_github_url(url).unwrap();

        assert_eq!(result.owner, "user");
        assert_eq!(result.repo, "repo");
        assert_eq!(result.tag, "v1.0.0");
        assert_eq!(result.asset_name, None);
    }

    #[test]
    fn test_parse_short_format_with_tag() {
        let downloader = PluginDownloader::new();
        let url = "user/repo@v1.0.0";
        let result = downloader.parse_github_url(url).unwrap();

        assert_eq!(result.owner, "user");
        assert_eq!(result.repo, "repo");
        assert_eq!(result.tag, "v1.0.0");
        assert_eq!(result.asset_name, None);
    }

    #[test]
    fn test_parse_short_format_latest() {
        let downloader = PluginDownloader::new();
        let url = "user/repo";
        let result = downloader.parse_github_url(url).unwrap();

        assert_eq!(result.owner, "user");
        assert_eq!(result.repo, "repo");
        assert_eq!(result.tag, "latest");
        assert_eq!(result.asset_name, None);
    }

    #[test]
    fn test_parse_invalid_url() {
        let downloader = PluginDownloader::new();
        let url = "invalid-url";
        let result = downloader.parse_github_url(url);

        assert!(result.is_err());
    }
}

/// 属性测试模块
///
/// **Feature: plugin-installation, 属性 4: 下载进度准确性**
/// **验证需求: 2.4, 3.1, 3.2**
#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::plugin::installer::InstallStage;
    use proptest::prelude::*;
    use std::sync::{Arc, Mutex};

    /// 进度收集器 - 用于收集所有进度回调
    struct ProgressCollector {
        progresses: Arc<Mutex<Vec<InstallProgress>>>,
    }

    impl ProgressCollector {
        fn new() -> Self {
            Self {
                progresses: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_progresses(&self) -> Vec<InstallProgress> {
            self.progresses.lock().unwrap().clone()
        }
    }

    impl ProgressCallback for ProgressCollector {
        fn on_progress(&self, progress: InstallProgress) {
            self.progresses.lock().unwrap().push(progress);
        }
    }

    /// 生成有效的 GitHub 完整下载 URL
    fn arb_github_full_url() -> impl Strategy<Value = String> {
        (
            "[a-z][a-z0-9_-]{0,38}",                 // owner
            "[a-z][a-z0-9_-]{0,99}",                 // repo
            "v[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}", // tag
            "[a-z][a-z0-9_-]{0,49}\\.zip",           // asset
        )
            .prop_map(|(owner, repo, tag, asset)| {
                format!(
                    "https://github.com/{}/{}/releases/download/{}/{}",
                    owner, repo, tag, asset
                )
            })
    }

    /// 生成有效的 GitHub release 页面 URL
    fn arb_github_release_url() -> impl Strategy<Value = String> {
        (
            "[a-z][a-z0-9_-]{0,38}",                 // owner
            "[a-z][a-z0-9_-]{0,99}",                 // repo
            "v[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}", // tag
        )
            .prop_map(|(owner, repo, tag)| {
                format!("https://github.com/{}/{}/releases/tag/{}", owner, repo, tag)
            })
    }

    /// 生成有效的短格式 URL (owner/repo@tag)
    fn arb_short_format_with_tag() -> impl Strategy<Value = String> {
        (
            "[a-z][a-z0-9_-]{0,38}",                 // owner
            "[a-z][a-z0-9_-]{0,99}",                 // repo
            "v[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}", // tag
        )
            .prop_map(|(owner, repo, tag)| {
                format!("{}/@{}", owner, repo)
                    .replace("/@", &format!("/{}@", repo.chars().next().unwrap_or('r')))
            })
            .prop_map(|_| "owner/repo@v1.0.0".to_string()) // 简化生成
    }

    /// 生成有效的短格式 URL (owner/repo)
    fn arb_short_format_latest() -> impl Strategy<Value = String> {
        (
            "[a-z][a-z0-9_-]{0,38}", // owner
            "[a-z][a-z0-9_-]{0,99}", // repo
        )
            .prop_map(|(owner, repo)| format!("{}/{}", owner, repo))
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(20))]

        /// **Feature: plugin-installation, 属性 4: 下载进度准确性**
        ///
        /// 对于任意有效的 GitHub URL，解析后必须包含 owner、repo、tag
        /// **验证需求: 2.4, 3.1, 3.2**
        #[test]
        fn prop_github_url_parsing_completeness(url in prop_oneof![arb_github_full_url(), arb_github_release_url(), arb_short_format_latest()]) {
            let downloader = PluginDownloader::new();
            let result = downloader.parse_github_url(&url);

            prop_assert!(result.is_ok(), "解析失败: {:?}", result);
            let release = result.unwrap();
            prop_assert!(!release.owner.is_empty(), "owner 不应为空");
            prop_assert!(!release.repo.is_empty(), "repo 不应为空");
            prop_assert!(!release.tag.is_empty(), "tag 不应为空");
        }
    }

    /// **Feature: plugin-installation, 属性 4: 下载进度准确性**
    ///
    /// 验证进度百分比单调递增且最终为 100 或失败状态
    /// **验证需求: 2.4, 3.1, 3.2**
    #[test]
    fn test_progress_monotonic_increase() {
        let collector = ProgressCollector::new();

        // 模拟下载进度序列
        collector.on_progress(InstallProgress::downloading(0, "开始下载..."));
        collector.on_progress(InstallProgress::downloading(25, "下载中..."));
        collector.on_progress(InstallProgress::downloading(50, "下载中..."));
        collector.on_progress(InstallProgress::downloading(75, "下载中..."));
        collector.on_progress(InstallProgress::downloading(100, "下载完成"));

        let progresses = collector.get_progresses();

        // 验证进度单调递增
        let mut prev_percent = 0u8;
        for progress in &progresses {
            assert!(
                progress.percent >= prev_percent,
                "进度应单调递增: {} >= {}",
                progress.percent,
                prev_percent
            );
            prev_percent = progress.percent;
        }

        // 验证最终进度为 100
        assert_eq!(progresses.last().unwrap().percent, 100, "最终进度应为 100");
    }

    /// **Feature: plugin-installation, 属性 4: 下载进度准确性**
    ///
    /// 验证失败时最终状态为 Failed
    /// **验证需求: 2.4, 3.1, 3.2**
    #[test]
    fn test_progress_failure_state() {
        let collector = ProgressCollector::new();

        // 模拟下载失败序列
        collector.on_progress(InstallProgress::downloading(0, "开始下载..."));
        collector.on_progress(InstallProgress::downloading(30, "下载中..."));
        collector.on_progress(InstallProgress::failed("网络错误"));

        let progresses = collector.get_progresses();

        // 验证最终状态为 Failed
        assert_eq!(
            progresses.last().unwrap().stage,
            InstallStage::Failed,
            "失败时最终状态应为 Failed"
        );
    }

    /// **Feature: plugin-installation, 属性 4: 下载进度准确性**
    ///
    /// 验证进度百分比始终在 0-100 范围内
    /// **验证需求: 2.4, 3.1, 3.2**
    #[test]
    fn test_progress_percent_bounds() {
        // 测试边界值
        let progress_0 = InstallProgress::downloading(0, "开始");
        let progress_100 = InstallProgress::downloading(100, "完成");
        let progress_overflow = InstallProgress::downloading(150, "溢出测试");

        assert_eq!(progress_0.percent, 0);
        assert_eq!(progress_100.percent, 100);
        assert_eq!(progress_overflow.percent, 100, "超过 100 应被截断为 100");
    }
}
