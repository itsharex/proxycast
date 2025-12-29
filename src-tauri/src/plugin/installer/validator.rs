//! 插件包验证器
//!
//! 验证插件包格式和内容
//!
//! 主要功能：
//! - 验证包格式（zip/tar.gz）
//! - 验证清单文件必需字段
//! - 验证包完整性（校验和）

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

use super::types::{InstallError, PackageFormat};
use crate::plugin::PluginManifest;

/// 包验证器
///
/// 验证插件包格式和内容
pub struct PackageValidator;

impl PackageValidator {
    /// 创建新的验证器
    pub fn new() -> Self {
        Self
    }

    /// 验证包格式（zip/tar.gz）
    ///
    /// 检测文件格式，验证压缩包完整性
    /// _需求: 1.1_
    pub fn validate_format(&self, path: &Path) -> Result<PackageFormat, InstallError> {
        // 首先检查文件是否存在
        if !path.exists() {
            return Err(InstallError::InvalidPackage(format!(
                "文件不存在: {}",
                path.display()
            )));
        }

        // 检查文件大小
        let metadata = std::fs::metadata(path)?;
        if metadata.len() == 0 {
            return Err(InstallError::InvalidPackage("文件为空".to_string()));
        }

        // 从扩展名检测格式
        let format = PackageFormat::from_extension(path).ok_or_else(|| {
            InstallError::InvalidPackage(format!(
                "不支持的包格式，仅支持 .zip 和 .tar.gz: {}",
                path.display()
            ))
        })?;

        // 验证文件魔数
        self.validate_magic_bytes(path, format)?;

        // 验证压缩包完整性
        self.validate_archive_integrity(path, format)?;

        Ok(format)
    }

    /// 验证文件魔数
    fn validate_magic_bytes(&self, path: &Path, format: PackageFormat) -> Result<(), InstallError> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut magic = [0u8; 4];

        reader
            .read_exact(&mut magic)
            .map_err(|e| InstallError::InvalidPackage(format!("无法读取文件头: {}", e)))?;

        match format {
            PackageFormat::Zip => {
                // ZIP 文件魔数: PK\x03\x04 (正常文件) 或 PK\x05\x06 (空压缩包)
                if magic[0..2] != [0x50, 0x4B] {
                    return Err(InstallError::InvalidPackage(
                        "无效的 ZIP 文件格式".to_string(),
                    ));
                }
            }
            PackageFormat::TarGz => {
                // Gzip 文件魔数: \x1f\x8b
                if magic[0..2] != [0x1f, 0x8b] {
                    return Err(InstallError::InvalidPackage(
                        "无效的 tar.gz 文件格式".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// 验证压缩包完整性
    ///
    /// 尝试读取压缩包结构，确保文件未损坏
    fn validate_archive_integrity(
        &self,
        path: &Path,
        format: PackageFormat,
    ) -> Result<(), InstallError> {
        match format {
            PackageFormat::Zip => self.validate_zip_integrity(path),
            PackageFormat::TarGz => self.validate_targz_integrity(path),
        }
    }

    /// 验证 ZIP 文件完整性
    fn validate_zip_integrity(&self, path: &Path) -> Result<(), InstallError> {
        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| InstallError::InvalidPackage(format!("无法读取 ZIP 文件: {}", e)))?;

        // 检查是否为空压缩包
        if archive.len() == 0 {
            return Err(InstallError::InvalidPackage("ZIP 压缩包为空".to_string()));
        }

        // 尝试读取每个文件的元数据以验证完整性
        for i in 0..archive.len() {
            let file = archive.by_index(i).map_err(|e| {
                InstallError::InvalidPackage(format!("ZIP 文件损坏，无法读取条目 {}: {}", i, e))
            })?;

            // 验证文件名有效
            if file.name().is_empty() {
                return Err(InstallError::InvalidPackage(format!(
                    "ZIP 条目 {} 的文件名无效",
                    i
                )));
            }
        }

        Ok(())
    }

    /// 验证 tar.gz 文件完整性
    fn validate_targz_integrity(&self, path: &Path) -> Result<(), InstallError> {
        let file = File::open(path)?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);

        let mut entry_count = 0;
        for entry in archive
            .entries()
            .map_err(|e| InstallError::InvalidPackage(format!("无法读取 tar.gz 文件: {}", e)))?
        {
            let entry = entry
                .map_err(|e| InstallError::InvalidPackage(format!("tar.gz 文件损坏: {}", e)))?;

            // 验证路径有效
            let path = entry
                .path()
                .map_err(|e| InstallError::InvalidPackage(format!("tar.gz 条目路径无效: {}", e)))?;

            if path.to_string_lossy().is_empty() {
                return Err(InstallError::InvalidPackage(
                    "tar.gz 条目路径为空".to_string(),
                ));
            }

            entry_count += 1;
        }

        if entry_count == 0 {
            return Err(InstallError::InvalidPackage(
                "tar.gz 压缩包为空".to_string(),
            ));
        }

        Ok(())
    }

    /// 验证清单文件
    ///
    /// 验证 plugin.json 必需字段
    /// _需求: 5.1, 5.2, 5.3, 5.4_
    pub fn validate_manifest(&self, manifest: &PluginManifest) -> Result<(), InstallError> {
        // 验证必需字段: name (5.1)
        if manifest.name.is_empty() {
            return Err(InstallError::InvalidManifest(
                "插件名称 (name) 不能为空".to_string(),
            ));
        }

        // 验证必需字段: version (5.1)
        if manifest.version.is_empty() {
            return Err(InstallError::InvalidManifest(
                "插件版本 (version) 不能为空".to_string(),
            ));
        }

        // 验证名称格式（只允许字母、数字、连字符、下划线）
        if !Self::is_valid_name(&manifest.name) {
            return Err(InstallError::InvalidManifest(
                "插件名称只能包含字母、数字、连字符和下划线".to_string(),
            ));
        }

        // 验证名称长度
        if manifest.name.len() > 64 {
            return Err(InstallError::InvalidManifest(
                "插件名称长度不能超过 64 个字符".to_string(),
            ));
        }

        // 验证版本格式（简单的 semver 检查）
        if !Self::is_valid_version(&manifest.version) {
            return Err(InstallError::InvalidManifest(format!(
                "无效的版本格式: {}，期望 semver 格式如 1.0.0",
                manifest.version
            )));
        }

        // 验证 plugin_type (5.2) - 类型已通过 serde 反序列化验证

        // 验证 entry 字段 (5.3)
        if manifest.entry.is_empty() {
            return Err(InstallError::InvalidManifest(
                "入口文件 (entry) 不能为空".to_string(),
            ));
        }

        // 验证 hooks 字段格式 (5.3)
        for hook in &manifest.hooks {
            if !Self::is_valid_hook_name(hook) {
                return Err(InstallError::InvalidManifest(format!(
                    "无效的钩子名称: {}",
                    hook
                )));
            }
        }

        Ok(())
    }

    /// 验证名称格式
    pub fn is_valid_name(name: &str) -> bool {
        !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    }

    /// 验证版本格式
    pub fn is_valid_version(version: &str) -> bool {
        if version.is_empty() {
            return false;
        }

        // 简单的 semver 验证：允许 x.y.z 或 x.y.z-suffix 格式
        let parts: Vec<&str> = version.split('-').collect();
        let version_part = parts[0];

        let numbers: Vec<&str> = version_part.split('.').collect();
        if numbers.len() < 2 || numbers.len() > 3 {
            return false;
        }

        numbers.iter().all(|n| n.parse::<u32>().is_ok())
    }

    /// 验证钩子名称格式
    fn is_valid_hook_name(hook: &str) -> bool {
        !hook.is_empty()
            && hook
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == ':')
    }

    /// 验证包完整性（校验和）
    ///
    /// 如果提供了校验和，验证文件的 SHA256 哈希
    pub fn validate_integrity(
        &self,
        path: &Path,
        checksum: Option<&str>,
    ) -> Result<(), InstallError> {
        let Some(expected) = checksum else {
            // 没有提供校验和，跳过验证
            return Ok(());
        };

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = sha2::Sha256::new();
        let mut buffer = [0u8; 8192];

        use sha2::Digest;
        loop {
            let bytes_read = reader.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let actual = format!("{:x}", hasher.finalize());

        if actual != expected.to_lowercase() {
            return Err(InstallError::ChecksumMismatch {
                expected: expected.to_string(),
                actual,
            });
        }

        Ok(())
    }

    /// 从压缩包中提取并验证清单
    ///
    /// 读取压缩包中的 plugin.json 并验证
    pub fn extract_and_validate_manifest(
        &self,
        path: &Path,
        format: PackageFormat,
    ) -> Result<PluginManifest, InstallError> {
        let manifest_content = match format {
            PackageFormat::Zip => self.extract_manifest_from_zip(path)?,
            PackageFormat::TarGz => self.extract_manifest_from_targz(path)?,
        };

        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| InstallError::InvalidManifest(format!("plugin.json 解析失败: {}", e)))?;

        self.validate_manifest(&manifest)?;

        Ok(manifest)
    }

    /// 从 ZIP 中提取 plugin.json
    fn extract_manifest_from_zip(&self, path: &Path) -> Result<String, InstallError> {
        let file = File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)
            .map_err(|e| InstallError::InvalidPackage(format!("无法读取 ZIP 文件: {}", e)))?;

        // 查找 plugin.json（可能在根目录或子目录中）
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| InstallError::InvalidPackage(format!("无法读取 ZIP 条目: {}", e)))?;

            let name = file.name().to_string();
            if name.ends_with("plugin.json") && !name.contains("__MACOSX") {
                let mut content = String::new();
                file.read_to_string(&mut content).map_err(|e| {
                    InstallError::InvalidManifest(format!("无法读取 plugin.json: {}", e))
                })?;
                return Ok(content);
            }
        }

        Err(InstallError::InvalidPackage(
            "压缩包中未找到 plugin.json".to_string(),
        ))
    }

    /// 从 tar.gz 中提取 plugin.json
    fn extract_manifest_from_targz(&self, path: &Path) -> Result<String, InstallError> {
        let file = File::open(path)?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);

        for entry in archive
            .entries()
            .map_err(|e| InstallError::InvalidPackage(format!("无法读取 tar.gz 文件: {}", e)))?
        {
            let mut entry = entry
                .map_err(|e| InstallError::InvalidPackage(format!("tar.gz 条目读取失败: {}", e)))?;

            let entry_path = entry
                .path()
                .map_err(|e| InstallError::InvalidPackage(format!("tar.gz 条目路径无效: {}", e)))?;

            if entry_path.ends_with("plugin.json") {
                let mut content = String::new();
                entry.read_to_string(&mut content).map_err(|e| {
                    InstallError::InvalidManifest(format!("无法读取 plugin.json: {}", e))
                })?;
                return Ok(content);
            }
        }

        Err(InstallError::InvalidPackage(
            "压缩包中未找到 plugin.json".to_string(),
        ))
    }
}

impl Default for PackageValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::PluginType;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_manifest(name: &str, version: &str) -> PluginManifest {
        PluginManifest {
            name: name.to_string(),
            version: version.to_string(),
            description: "Test plugin".to_string(),
            author: None,
            homepage: None,
            license: None,
            entry: "config.json".to_string(),
            plugin_type: PluginType::Script,
            config_schema: None,
            hooks: vec![],
            min_proxycast_version: None,
            binary: None,
            ui: None,
        }
    }

    #[test]
    fn test_validate_manifest_valid() {
        let validator = PackageValidator::new();
        let manifest = create_test_manifest("test-plugin", "1.0.0");
        assert!(validator.validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn test_validate_manifest_empty_name() {
        let validator = PackageValidator::new();
        let manifest = create_test_manifest("", "1.0.0");
        let result = validator.validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("名称"));
    }

    #[test]
    fn test_validate_manifest_empty_version() {
        let validator = PackageValidator::new();
        let manifest = create_test_manifest("test-plugin", "");
        let result = validator.validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("版本"));
    }

    #[test]
    fn test_validate_manifest_invalid_name() {
        let validator = PackageValidator::new();
        let manifest = create_test_manifest("test plugin!", "1.0.0");
        let result = validator.validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_manifest_invalid_version() {
        let validator = PackageValidator::new();
        let manifest = create_test_manifest("test-plugin", "invalid");
        let result = validator.validate_manifest(&manifest);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_manifest_valid_versions() {
        let validator = PackageValidator::new();

        // 有效版本格式
        let valid_versions = ["1.0", "1.0.0", "0.1.0", "10.20.30", "1.0.0-beta"];

        for version in valid_versions {
            let manifest = create_test_manifest("test-plugin", version);
            assert!(
                validator.validate_manifest(&manifest).is_ok(),
                "Version {} should be valid",
                version
            );
        }
    }

    #[test]
    fn test_validate_manifest_name_too_long() {
        let validator = PackageValidator::new();
        let long_name = "a".repeat(65);
        let manifest = create_test_manifest(&long_name, "1.0.0");
        let result = validator.validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("64"));
    }

    #[test]
    fn test_validate_manifest_empty_entry() {
        let validator = PackageValidator::new();
        let mut manifest = create_test_manifest("test-plugin", "1.0.0");
        manifest.entry = "".to_string();
        let result = validator.validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("入口"));
    }

    #[test]
    fn test_validate_manifest_invalid_hook() {
        let validator = PackageValidator::new();
        let mut manifest = create_test_manifest("test-plugin", "1.0.0");
        manifest.hooks = vec!["valid_hook".to_string(), "invalid hook!".to_string()];
        let result = validator.validate_manifest(&manifest);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("钩子"));
    }

    #[test]
    fn test_is_valid_name() {
        assert!(PackageValidator::is_valid_name("test-plugin"));
        assert!(PackageValidator::is_valid_name("test_plugin"));
        assert!(PackageValidator::is_valid_name("TestPlugin123"));
        assert!(!PackageValidator::is_valid_name(""));
        assert!(!PackageValidator::is_valid_name("test plugin"));
        assert!(!PackageValidator::is_valid_name("test.plugin"));
    }

    #[test]
    fn test_is_valid_version() {
        assert!(PackageValidator::is_valid_version("1.0"));
        assert!(PackageValidator::is_valid_version("1.0.0"));
        assert!(PackageValidator::is_valid_version("1.0.0-beta"));
        assert!(PackageValidator::is_valid_version("0.1.0"));
        assert!(!PackageValidator::is_valid_version(""));
        assert!(!PackageValidator::is_valid_version("invalid"));
        assert!(!PackageValidator::is_valid_version("1"));
        assert!(!PackageValidator::is_valid_version("1.0.0.0"));
    }

    #[test]
    fn test_validate_format_nonexistent_file() {
        let validator = PackageValidator::new();
        let result = validator.validate_format(Path::new("/nonexistent/file.zip"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不存在"));
    }

    #[test]
    fn test_validate_format_unsupported_extension() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_format(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("不支持"));
    }

    #[test]
    fn test_validate_format_invalid_zip_magic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.zip");
        std::fs::write(&file_path, "not a zip file").unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_format(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无效的 ZIP"));
    }

    #[test]
    fn test_validate_format_invalid_targz_magic() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.tar.gz");
        std::fs::write(&file_path, "not a tar.gz file").unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_format(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("无效的 tar.gz"));
    }

    #[test]
    fn test_validate_format_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("empty.zip");
        std::fs::write(&file_path, "").unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_format(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("为空"));
    }

    #[test]
    fn test_validate_format_valid_zip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.zip");

        // 创建有效的 ZIP 文件
        let file = File::create(&file_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();
        zip.start_file("test.txt", options).unwrap();
        zip.write_all(b"test content").unwrap();
        zip.finish().unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_format(&file_path);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), PackageFormat::Zip);
    }

    #[test]
    fn test_validate_format_valid_targz() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.tar.gz");

        // 创建有效的 tar.gz 文件
        let file = File::create(&file_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);

        let content = b"test content";
        let mut header = tar::Header::new_gnu();
        header.set_path("test.txt").unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, &content[..]).unwrap();
        tar.into_inner().unwrap().finish().unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_format(&file_path);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        assert_eq!(result.unwrap(), PackageFormat::TarGz);
    }

    #[test]
    fn test_extract_manifest_from_zip() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("plugin.zip");

        // 创建包含 plugin.json 的 ZIP 文件
        let manifest_json = r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "Test plugin",
            "entry": "config.json",
            "plugin_type": "script",
            "hooks": []
        }"#;

        let file = File::create(&file_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();
        zip.start_file("plugin.json", options).unwrap();
        zip.write_all(manifest_json.as_bytes()).unwrap();
        zip.finish().unwrap();

        let validator = PackageValidator::new();
        let result = validator.extract_and_validate_manifest(&file_path, PackageFormat::Zip);
        assert!(result.is_ok());
        let manifest = result.unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
    }

    #[test]
    fn test_extract_manifest_from_targz() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("plugin.tar.gz");

        // 创建包含 plugin.json 的 tar.gz 文件
        let manifest_json = r#"{
            "name": "test-plugin",
            "version": "1.0.0",
            "description": "Test plugin",
            "entry": "config.json",
            "plugin_type": "script",
            "hooks": []
        }"#;

        let file = File::create(&file_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);

        let content = manifest_json.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("plugin.json").unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, content).unwrap();
        tar.into_inner().unwrap().finish().unwrap();

        let validator = PackageValidator::new();
        let result = validator.extract_and_validate_manifest(&file_path, PackageFormat::TarGz);
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let manifest = result.unwrap();
        assert_eq!(manifest.name, "test-plugin");
        assert_eq!(manifest.version, "1.0.0");
    }

    #[test]
    fn test_extract_manifest_missing() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("no-manifest.zip");

        // 创建不包含 plugin.json 的 ZIP 文件
        let file = File::create(&file_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();
        zip.start_file("other.txt", options).unwrap();
        zip.write_all(b"other content").unwrap();
        zip.finish().unwrap();

        let validator = PackageValidator::new();
        let result = validator.extract_and_validate_manifest(&file_path, PackageFormat::Zip);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("plugin.json"));
    }

    #[test]
    fn test_validate_integrity_no_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_integrity(&file_path, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_integrity_valid_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        // SHA256 of "test content"
        let checksum = "6ae8a75555209fd6c44157c0aed8016e763ff435a19cf186f76863140143ff72";

        let validator = PackageValidator::new();
        let result = validator.validate_integrity(&file_path, Some(checksum));
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_integrity_invalid_checksum() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "test content").unwrap();

        let validator = PackageValidator::new();
        let result = validator.validate_integrity(&file_path, Some("invalid_checksum"));
        assert!(result.is_err());
        match result.unwrap_err() {
            InstallError::ChecksumMismatch {
                expected,
                actual: _,
            } => {
                assert_eq!(expected, "invalid_checksum");
            }
            _ => panic!("Expected ChecksumMismatch error"),
        }
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::plugin::PluginType;
    use proptest::prelude::*;
    use std::io::Write;
    use tempfile::TempDir;

    /// 生成有效的插件名称
    fn arb_valid_plugin_name() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_-]{0,30}".prop_map(|s| s)
    }

    /// 生成有效的版本号
    fn arb_valid_version() -> impl Strategy<Value = String> {
        (1u32..100, 0u32..100, 0u32..100)
            .prop_map(|(major, minor, patch)| format!("{}.{}.{}", major, minor, patch))
    }

    /// 生成有效的插件类型
    fn arb_plugin_type() -> impl Strategy<Value = &'static str> {
        prop_oneof![Just("script"), Just("native"), Just("binary"),]
    }

    /// 生成有效的钩子名称
    fn arb_valid_hook() -> impl Strategy<Value = String> {
        "[a-zA-Z][a-zA-Z0-9_:]{0,20}".prop_map(|s| s)
    }

    /// 生成有效的 PluginManifest JSON
    fn arb_valid_manifest_json() -> impl Strategy<Value = String> {
        (
            arb_valid_plugin_name(),
            arb_valid_version(),
            arb_plugin_type(),
            prop::collection::vec(arb_valid_hook(), 0..3),
            any::<Option<String>>().prop_map(|opt| opt.map(|_| "Test description".to_string())),
        )
            .prop_map(|(name, version, plugin_type, hooks, description)| {
                let hooks_json = hooks
                    .iter()
                    .map(|h| format!("\"{}\"", h))
                    .collect::<Vec<_>>()
                    .join(", ");
                let desc = description.unwrap_or_else(|| "Test plugin".to_string());
                format!(
                    r#"{{
                    "name": "{}",
                    "version": "{}",
                    "description": "{}",
                    "entry": "config.json",
                    "plugin_type": "{}",
                    "hooks": [{}]
                }}"#,
                    name, version, desc, plugin_type, hooks_json
                )
            })
    }

    /// 创建包含指定 manifest 的 ZIP 文件
    fn create_zip_with_manifest(temp_dir: &TempDir, manifest_json: &str) -> std::path::PathBuf {
        let file_path = temp_dir.path().join("plugin.zip");
        let file = File::create(&file_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::FileOptions::default();
        zip.start_file("plugin.json", options).unwrap();
        zip.write_all(manifest_json.as_bytes()).unwrap();
        zip.finish().unwrap();
        file_path
    }

    /// 创建包含指定 manifest 的 tar.gz 文件
    fn create_targz_with_manifest(temp_dir: &TempDir, manifest_json: &str) -> std::path::PathBuf {
        let file_path = temp_dir.path().join("plugin.tar.gz");
        let file = File::create(&file_path).unwrap();
        let gz = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut tar = tar::Builder::new(gz);

        let content = manifest_json.as_bytes();
        let mut header = tar::Header::new_gnu();
        header.set_path("plugin.json").unwrap();
        header.set_size(content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append(&header, content).unwrap();
        tar.into_inner().unwrap().finish().unwrap();
        file_path
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Feature: plugin-installation, 属性 1: 包验证完整性
        ///
        /// *对于任意*插件包文件，如果包通过验证，则它必须包含有效的 plugin.json 清单，
        /// 且包含所有必需字段（name、version、plugin_type）。
        ///
        /// **验证需求: 1.1, 5.1, 5.4**
        #[test]
        fn prop_valid_zip_package_contains_valid_manifest(manifest_json in arb_valid_manifest_json()) {
            let temp_dir = TempDir::new().unwrap();
            let file_path = create_zip_with_manifest(&temp_dir, &manifest_json);

            let validator = PackageValidator::new();

            // 验证包格式
            let format_result = validator.validate_format(&file_path);
            prop_assert!(format_result.is_ok(), "Format validation failed: {:?}", format_result);
            prop_assert_eq!(format_result.unwrap(), PackageFormat::Zip);

            // 提取并验证清单
            let manifest_result = validator.extract_and_validate_manifest(&file_path, PackageFormat::Zip);
            prop_assert!(manifest_result.is_ok(), "Manifest validation failed: {:?}", manifest_result);

            let manifest = manifest_result.unwrap();

            // 验证必需字段存在且非空
            prop_assert!(!manifest.name.is_empty(), "Name should not be empty");
            prop_assert!(!manifest.version.is_empty(), "Version should not be empty");
            prop_assert!(!manifest.entry.is_empty(), "Entry should not be empty");

            // 验证名称格式
            prop_assert!(
                PackageValidator::is_valid_name(&manifest.name),
                "Name should be valid: {}",
                manifest.name
            );

            // 验证版本格式
            prop_assert!(
                PackageValidator::is_valid_version(&manifest.version),
                "Version should be valid: {}",
                manifest.version
            );
        }

        /// Feature: plugin-installation, 属性 1: 包验证完整性 (tar.gz)
        ///
        /// **验证需求: 1.1, 5.1, 5.4**
        #[test]
        fn prop_valid_targz_package_contains_valid_manifest(manifest_json in arb_valid_manifest_json()) {
            let temp_dir = TempDir::new().unwrap();
            let file_path = create_targz_with_manifest(&temp_dir, &manifest_json);

            let validator = PackageValidator::new();

            // 验证包格式
            let format_result = validator.validate_format(&file_path);
            prop_assert!(format_result.is_ok(), "Format validation failed: {:?}", format_result);
            prop_assert_eq!(format_result.unwrap(), PackageFormat::TarGz);

            // 提取并验证清单
            let manifest_result = validator.extract_and_validate_manifest(&file_path, PackageFormat::TarGz);
            prop_assert!(manifest_result.is_ok(), "Manifest validation failed: {:?}", manifest_result);

            let manifest = manifest_result.unwrap();

            // 验证必需字段存在且非空
            prop_assert!(!manifest.name.is_empty(), "Name should not be empty");
            prop_assert!(!manifest.version.is_empty(), "Version should not be empty");
            prop_assert!(!manifest.entry.is_empty(), "Entry should not be empty");

            // 验证名称格式
            prop_assert!(
                PackageValidator::is_valid_name(&manifest.name),
                "Name should be valid: {}",
                manifest.name
            );

            // 验证版本格式
            prop_assert!(
                PackageValidator::is_valid_version(&manifest.version),
                "Version should be valid: {}",
                manifest.version
            );
        }

        /// Feature: plugin-installation, 属性 1: 包验证完整性 (反向测试)
        ///
        /// *对于任意*无效的清单（缺少必需字段），验证应该失败。
        ///
        /// **验证需求: 5.4**
        #[test]
        fn prop_invalid_manifest_fails_validation(
            name in prop::option::of(arb_valid_plugin_name()),
            version in prop::option::of(arb_valid_version()),
        ) {
            // 只有当 name 或 version 缺失时才测试
            prop_assume!(name.is_none() || version.is_none());

            let manifest = PluginManifest {
                name: name.unwrap_or_default(),
                version: version.unwrap_or_default(),
                description: "Test".to_string(),
                author: None,
                homepage: None,
                license: None,
                entry: "config.json".to_string(),
                plugin_type: PluginType::Script,
                config_schema: None,
                hooks: vec![],
                min_proxycast_version: None,
                binary: None,
                ui: None,
            };

            let validator = PackageValidator::new();
            let result = validator.validate_manifest(&manifest);

            // 缺少必需字段时验证应该失败
            prop_assert!(result.is_err(), "Validation should fail for invalid manifest");
        }
    }
}
