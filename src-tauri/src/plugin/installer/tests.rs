//! 插件安装器测试
//!
//! 测试插件安装、注册、加载的完整流程

use super::*;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use zip::write::FileOptions;
use zip::ZipWriter;

/// 创建测试用的插件包
fn create_test_plugin_zip(dir: &TempDir, plugin_id: &str, version: &str) -> std::path::PathBuf {
    let zip_path = dir.path().join(format!("{plugin_id}.zip"));
    let file = fs::File::create(&zip_path).unwrap();
    let mut zip = ZipWriter::new(file);

    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // 创建 plugin.json
    let manifest = serde_json::json!({
        "name": plugin_id,
        "version": version,
        "description": format!("Test plugin {}", plugin_id),
        "author": "Test Author",
        "plugin_type": "script",
        "entry": "config.json",
        "ui": {
            "surfaces": ["tools"],
            "entry": "dist/index.js",
            "icon": "Package"
        }
    });

    zip.start_file("plugin.json", options).unwrap();
    zip.write_all(manifest.to_string().as_bytes()).unwrap();

    // 创建 config.json
    let config = serde_json::json!({
        "enabled": true
    });
    zip.start_file("config.json", options).unwrap();
    zip.write_all(config.to_string().as_bytes()).unwrap();

    // 创建 dist/index.js
    zip.start_file("dist/index.js", options).unwrap();
    zip.write_all(b"// Plugin UI code").unwrap();

    zip.finish().unwrap();
    zip_path
}

/// 创建测试注册表
fn create_test_registry(dir: &TempDir) -> PluginRegistry {
    let db_path = dir.path().join("test.db");
    let registry = PluginRegistry::from_path(&db_path).unwrap();
    registry.init_tables().unwrap();
    registry
}

#[cfg(test)]
mod registry_tests {
    use super::*;

    #[test]
    fn test_registry_create_and_register() {
        let temp_dir = TempDir::new().unwrap();
        let registry = create_test_registry(&temp_dir);

        let plugin = InstalledPlugin {
            id: "test-plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: Some("Test Author".to_string()),
            install_path: temp_dir.path().join("test-plugin"),
            installed_at: chrono::Utc::now(),
            source: InstallSource::Local {
                path: "/tmp/test.zip".to_string(),
            },
            enabled: true,
        };

        // 注册插件
        registry.register(&plugin).unwrap();

        // 验证插件存在
        assert!(registry.exists("test-plugin").unwrap());
        assert!(!registry.exists("non-existent").unwrap());
    }

    #[test]
    fn test_registry_get_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let registry = create_test_registry(&temp_dir);

        let plugin = InstalledPlugin {
            id: "get-test".to_string(),
            name: "Get Test Plugin".to_string(),
            version: "2.0.0".to_string(),
            description: "Plugin for get test".to_string(),
            author: None,
            install_path: temp_dir.path().join("get-test"),
            installed_at: chrono::Utc::now(),
            source: InstallSource::Url {
                url: "https://example.com/plugin.zip".to_string(),
            },
            enabled: false,
        };

        registry.register(&plugin).unwrap();

        // 获取插件
        let retrieved = registry.get("get-test").unwrap().unwrap();
        assert_eq!(retrieved.id, "get-test");
        assert_eq!(retrieved.name, "Get Test Plugin");
        assert_eq!(retrieved.version, "2.0.0");
        assert!(!retrieved.enabled);

        // 获取不存在的插件
        let not_found = registry.get("non-existent").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_registry_list_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let registry = create_test_registry(&temp_dir);

        // 注册多个插件
        for i in 1..=3 {
            let plugin = InstalledPlugin {
                id: format!("plugin-{i}"),
                name: format!("Plugin {i}"),
                version: "1.0.0".to_string(),
                description: format!("Plugin {i} description"),
                author: Some("Author".to_string()),
                install_path: temp_dir.path().join(format!("plugin-{i}")),
                installed_at: chrono::Utc::now(),
                source: InstallSource::Local {
                    path: format!("/tmp/plugin-{i}.zip"),
                },
                enabled: true,
            };
            registry.register(&plugin).unwrap();
        }

        // 列出所有插件
        let plugins = registry.list().unwrap();
        assert_eq!(plugins.len(), 3);
    }

    #[test]
    fn test_registry_unregister_plugin() {
        let temp_dir = TempDir::new().unwrap();
        let registry = create_test_registry(&temp_dir);

        let plugin = InstalledPlugin {
            id: "unregister-test".to_string(),
            name: "Unregister Test".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            author: None,
            install_path: temp_dir.path().join("unregister-test"),
            installed_at: chrono::Utc::now(),
            source: InstallSource::Local {
                path: "/tmp/test.zip".to_string(),
            },
            enabled: true,
        };

        registry.register(&plugin).unwrap();
        assert!(registry.exists("unregister-test").unwrap());

        // 注销插件
        registry.unregister("unregister-test").unwrap();
        assert!(!registry.exists("unregister-test").unwrap());

        // 注销不存在的插件应该返回错误
        let result = registry.unregister("non-existent");
        assert!(result.is_err());
    }

    #[test]
    fn test_registry_set_enabled() {
        let temp_dir = TempDir::new().unwrap();
        let registry = create_test_registry(&temp_dir);

        let plugin = InstalledPlugin {
            id: "enabled-test".to_string(),
            name: "Enabled Test".to_string(),
            version: "1.0.0".to_string(),
            description: "".to_string(),
            author: None,
            install_path: temp_dir.path().join("enabled-test"),
            installed_at: chrono::Utc::now(),
            source: InstallSource::Local {
                path: "/tmp/test.zip".to_string(),
            },
            enabled: true,
        };

        registry.register(&plugin).unwrap();

        // 禁用插件
        registry.set_enabled("enabled-test", false).unwrap();
        let retrieved = registry.get("enabled-test").unwrap().unwrap();
        assert!(!retrieved.enabled);

        // 启用插件
        registry.set_enabled("enabled-test", true).unwrap();
        let retrieved = registry.get("enabled-test").unwrap().unwrap();
        assert!(retrieved.enabled);
    }
}

#[cfg(test)]
mod installer_tests {
    use super::*;

    #[test]
    fn test_installer_from_paths() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        let installer =
            PluginInstaller::from_paths(plugins_dir.clone(), temp_install_dir.clone(), &db_path)
                .unwrap();

        assert_eq!(installer.plugins_dir(), plugins_dir);
        assert_eq!(installer.temp_dir(), temp_install_dir);
    }

    #[test]
    fn test_installer_list_installed_empty() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        let installer =
            PluginInstaller::from_paths(plugins_dir, temp_install_dir, &db_path).unwrap();

        let plugins = installer.list_installed().unwrap();
        assert!(plugins.is_empty());
    }

    #[test]
    fn test_installer_is_installed() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        let installer =
            PluginInstaller::from_paths(plugins_dir, temp_install_dir, &db_path).unwrap();

        // 未安装的插件
        assert!(!installer.is_installed("non-existent").unwrap());
    }

    #[tokio::test]
    async fn test_installer_install_from_file() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        // 创建测试插件包
        let zip_path = create_test_plugin_zip(&temp_dir, "local-test-plugin", "1.0.0");

        let installer =
            PluginInstaller::from_paths(plugins_dir.clone(), temp_install_dir, &db_path).unwrap();

        // 安装插件
        let progress = NoopProgressCallback;
        let result = installer.install_from_file(&zip_path, &progress).await;

        assert!(result.is_ok(), "Install failed: {:?}", result.err());

        let installed = result.unwrap();
        assert_eq!(installed.id, "local-test-plugin");
        assert_eq!(installed.version, "1.0.0");
        assert!(installed.enabled);

        // 验证插件已注册
        assert!(installer.is_installed("local-test-plugin").unwrap());

        // 验证插件文件已复制
        let plugin_dir = plugins_dir.join("local-test-plugin");
        assert!(plugin_dir.exists());
        assert!(plugin_dir.join("plugin.json").exists());
        assert!(plugin_dir.join("config.json").exists());
        assert!(plugin_dir.join("dist/index.js").exists());
    }

    #[tokio::test]
    async fn test_installer_uninstall() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        // 创建并安装测试插件
        let zip_path = create_test_plugin_zip(&temp_dir, "uninstall-test", "1.0.0");

        let installer =
            PluginInstaller::from_paths(plugins_dir.clone(), temp_install_dir, &db_path).unwrap();

        let progress = NoopProgressCallback;
        installer
            .install_from_file(&zip_path, &progress)
            .await
            .unwrap();

        // 验证已安装
        assert!(installer.is_installed("uninstall-test").unwrap());
        assert!(plugins_dir.join("uninstall-test").exists());

        // 卸载插件
        installer.uninstall("uninstall-test").await.unwrap();

        // 验证已卸载
        assert!(!installer.is_installed("uninstall-test").unwrap());
        assert!(!plugins_dir.join("uninstall-test").exists());
    }

    #[tokio::test]
    async fn test_installer_reinstall_updates_version() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        let installer =
            PluginInstaller::from_paths(plugins_dir, temp_install_dir, &db_path).unwrap();

        let progress = NoopProgressCallback;

        // 安装 v1.0.0
        let zip_v1 = create_test_plugin_zip(&temp_dir, "version-test", "1.0.0");
        let installed_v1 = installer
            .install_from_file(&zip_v1, &progress)
            .await
            .unwrap();
        assert_eq!(installed_v1.version, "1.0.0");

        // 安装 v2.0.0（覆盖安装）
        let zip_v2 = create_test_plugin_zip(&temp_dir, "version-test", "2.0.0");
        let installed_v2 = installer
            .install_from_file(&zip_v2, &progress)
            .await
            .unwrap();
        assert_eq!(installed_v2.version, "2.0.0");

        // 验证数据库中的版本已更新
        let plugin = installer.get_plugin("version-test").unwrap().unwrap();
        assert_eq!(plugin.version, "2.0.0");
    }
}

#[cfg(test)]
mod validator_tests {
    use super::*;

    #[test]
    fn test_validator_valid_package() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = create_test_plugin_zip(&temp_dir, "valid-plugin", "1.0.0");

        let validator = PackageValidator::new();

        // 验证格式
        let format = validator.validate_format(&zip_path);
        assert!(
            format.is_ok(),
            "Format validation failed: {:?}",
            format.err()
        );

        // 提取并验证 manifest
        let manifest = validator.extract_and_validate_manifest(&zip_path, format.unwrap());
        assert!(
            manifest.is_ok(),
            "Manifest validation failed: {:?}",
            manifest.err()
        );

        let manifest = manifest.unwrap();
        assert_eq!(manifest.name, "valid-plugin");
        assert_eq!(manifest.version, "1.0.0");
    }

    #[test]
    fn test_validator_missing_manifest() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("no-manifest.zip");

        // 创建没有 plugin.json 的 zip
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();
        zip.start_file("config.json", options).unwrap();
        zip.write_all(b"{}").unwrap();
        zip.finish().unwrap();

        let validator = PackageValidator::new();
        let format = validator.validate_format(&zip_path).unwrap();
        let result = validator.extract_and_validate_manifest(&zip_path, format);

        assert!(result.is_err());
    }

    #[test]
    fn test_validator_invalid_manifest_json() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("invalid-json.zip");

        // 创建有无效 JSON 的 zip
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();
        zip.start_file("plugin.json", options).unwrap();
        zip.write_all(b"{ invalid json }").unwrap();
        zip.finish().unwrap();

        let validator = PackageValidator::new();
        let format = validator.validate_format(&zip_path).unwrap();
        let result = validator.extract_and_validate_manifest(&zip_path, format);

        assert!(result.is_err());
    }

    #[test]
    fn test_validator_missing_required_fields() {
        let temp_dir = TempDir::new().unwrap();
        let zip_path = temp_dir.path().join("missing-fields.zip");

        // 创建缺少必填字段的 manifest
        let file = fs::File::create(&zip_path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = FileOptions::default();
        zip.start_file("plugin.json", options).unwrap();
        // 缺少 name 和 version
        zip.write_all(b"{\"description\": \"test\"}").unwrap();
        zip.finish().unwrap();

        let validator = PackageValidator::new();
        let format = validator.validate_format(&zip_path).unwrap();
        let result = validator.extract_and_validate_manifest(&zip_path, format);

        assert!(result.is_err());
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// 测试完整的安装-加载-卸载流程
    #[tokio::test]
    async fn test_full_plugin_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        // 创建测试插件包
        let zip_path = create_test_plugin_zip(&temp_dir, "lifecycle-test", "1.0.0");

        let installer =
            PluginInstaller::from_paths(plugins_dir.clone(), temp_install_dir, &db_path).unwrap();

        let progress = NoopProgressCallback;

        // 1. 安装插件
        let installed = installer
            .install_from_file(&zip_path, &progress)
            .await
            .unwrap();
        assert_eq!(installed.id, "lifecycle-test");

        // 2. 验证插件已注册
        assert!(installer.is_installed("lifecycle-test").unwrap());

        // 3. 获取插件信息
        let plugin_info = installer.get_plugin("lifecycle-test").unwrap().unwrap();
        assert_eq!(plugin_info.version, "1.0.0");
        assert!(plugin_info.enabled);

        // 4. 验证插件文件存在
        let plugin_dir = plugins_dir.join("lifecycle-test");
        assert!(plugin_dir.join("plugin.json").exists());
        assert!(plugin_dir.join("dist/index.js").exists());

        // 5. 列出所有插件
        let all_plugins = installer.list_installed().unwrap();
        assert_eq!(all_plugins.len(), 1);
        assert_eq!(all_plugins[0].id, "lifecycle-test");

        // 6. 卸载插件
        installer.uninstall("lifecycle-test").await.unwrap();

        // 7. 验证插件已移除
        assert!(!installer.is_installed("lifecycle-test").unwrap());
        assert!(!plugin_dir.exists());

        // 8. 列表应为空
        let all_plugins = installer.list_installed().unwrap();
        assert!(all_plugins.is_empty());
    }

    /// 测试多个插件同时安装
    #[tokio::test]
    async fn test_multiple_plugins() {
        let temp_dir = TempDir::new().unwrap();
        let plugins_dir = temp_dir.path().join("plugins");
        let temp_install_dir = temp_dir.path().join("temp");
        let db_path = temp_dir.path().join("test.db");

        let installer =
            PluginInstaller::from_paths(plugins_dir, temp_install_dir, &db_path).unwrap();

        let progress = NoopProgressCallback;

        // 安装多个插件
        let plugin_ids = ["plugin-a", "plugin-b", "plugin-c"];
        for id in &plugin_ids {
            let zip_path = create_test_plugin_zip(&temp_dir, id, "1.0.0");
            installer
                .install_from_file(&zip_path, &progress)
                .await
                .unwrap();
        }

        // 验证所有插件都已安装
        let all_plugins = installer.list_installed().unwrap();
        assert_eq!(all_plugins.len(), 3);

        for id in &plugin_ids {
            assert!(installer.is_installed(id).unwrap());
        }

        // 卸载一个插件
        installer.uninstall("plugin-b").await.unwrap();

        // 验证只剩两个
        let remaining = installer.list_installed().unwrap();
        assert_eq!(remaining.len(), 2);
        assert!(!installer.is_installed("plugin-b").unwrap());
    }
}
