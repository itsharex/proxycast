//! API Key Provider 属性测试
//!
//! 使用 proptest 进行属性测试，验证 API Key Provider 服务的正确性。
//!
//! **Feature: provider-ui-refactor**

use proptest::prelude::*;
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::TempDir;

use proxycast_lib::database::dao::api_key_provider::{
    ApiKeyEntry, ApiKeyProvider, ApiKeyProviderDao, ApiProviderType, ProviderGroup,
};
use proxycast_lib::database::DbConnection;
use proxycast_lib::services::api_key_provider_service::ApiKeyProviderService;
use rusqlite::Connection;

/// 测试上下文
#[allow(dead_code)]
struct TestContext {
    pub temp_dir: TempDir,
    pub db: DbConnection,
    pub service: ApiKeyProviderService,
}

impl TestContext {
    /// 创建测试上下文
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path)?;

        // 创建表结构
        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_key_providers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                type TEXT NOT NULL,
                api_host TEXT NOT NULL,
                is_system INTEGER NOT NULL DEFAULT 0,
                group_name TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 0,
                sort_order INTEGER NOT NULL DEFAULT 0,
                api_version TEXT,
                project TEXT,
                location TEXT,
                region TEXT,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS api_keys (
                id TEXT PRIMARY KEY,
                provider_id TEXT NOT NULL,
                api_key_encrypted TEXT NOT NULL,
                alias TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                usage_count INTEGER NOT NULL DEFAULT 0,
                error_count INTEGER NOT NULL DEFAULT 0,
                last_used_at TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (provider_id) REFERENCES api_key_providers(id) ON DELETE CASCADE
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS provider_ui_state (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        let db = Arc::new(std::sync::Mutex::new(conn));
        let service = ApiKeyProviderService::new();

        Ok(Self {
            temp_dir,
            db,
            service,
        })
    }

    /// 创建测试 Provider
    pub fn create_test_provider(&self, id: &str) -> Result<ApiKeyProvider, String> {
        let now = chrono::Utc::now();
        let provider = ApiKeyProvider {
            id: id.to_string(),
            name: format!("Test Provider {id}"),
            provider_type: ApiProviderType::Openai,
            api_host: "https://api.test.com".to_string(),
            is_system: false,
            group: ProviderGroup::Custom,
            enabled: true,
            sort_order: 0,
            api_version: None,
            project: None,
            location: None,
            region: None,
            custom_models: vec![],
            created_at: now,
            updated_at: now,
        };

        let conn = self.db.lock().map_err(|e| e.to_string())?;
        ApiKeyProviderDao::insert_provider(&conn, &provider).map_err(|e| e.to_string())?;

        Ok(provider)
    }

    /// 添加测试 API Key
    pub fn add_test_api_key(
        &self,
        provider_id: &str,
        api_key: &str,
    ) -> Result<ApiKeyEntry, String> {
        self.service
            .add_api_key(&self.db, provider_id, api_key, None)
    }
}

// ============================================================================
// Property 12: 轮询负载均衡正确性
// **Validates: Requirements 7.3**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 12: 轮询负载均衡正确性
    ///
    /// *对于任意* 拥有 N 个启用的 API Key 的 Provider，连续 N 次获取 API Key 应各返回不同的 Key
    ///
    /// **Feature: provider-ui-refactor, Property 12: 轮询负载均衡正确性**
    /// **Validates: Requirements 7.3**
    #[test]
    fn test_round_robin_load_balancing(num_keys in 2usize..10) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建测试 Provider
        let provider_id = format!("test-provider-{}", uuid::Uuid::new_v4());
        ctx.create_test_provider(&provider_id).expect("Failed to create provider");

        // 添加 N 个 API Keys
        let mut expected_keys = Vec::new();
        for i in 0..num_keys {
            let api_key = format!("sk-test-key-{provider_id}-{i}");
            ctx.add_test_api_key(&provider_id, &api_key).expect("Failed to add API key");
            expected_keys.push(api_key);
        }

        // 连续获取 N 次 API Key
        let mut retrieved_keys = Vec::new();
        for _ in 0..num_keys {
            let key = ctx.service
                .get_next_api_key(&ctx.db, &provider_id)
                .expect("Failed to get next API key")
                .expect("No API key returned");
            retrieved_keys.push(key);
        }

        // 验证：连续 N 次获取应返回 N 个不同的 Key
        let unique_keys: HashSet<_> = retrieved_keys.iter().collect();
        prop_assert_eq!(
            unique_keys.len(),
            num_keys,
            "Expected {} unique keys, but got {}. Keys: {:?}",
            num_keys,
            unique_keys.len(),
            retrieved_keys
        );

        // 验证：所有返回的 Key 都在预期列表中
        for key in &retrieved_keys {
            prop_assert!(
                expected_keys.contains(key),
                "Unexpected key returned: {}",
                key
            );
        }
    }

    /// Property 12 补充测试：轮询循环性
    ///
    /// *对于任意* 拥有 N 个启用的 API Key 的 Provider，获取 2N 次应该循环使用所有 Key
    ///
    /// **Feature: provider-ui-refactor, Property 12: 轮询负载均衡正确性**
    /// **Validates: Requirements 7.3**
    #[test]
    fn test_round_robin_cycling(num_keys in 2usize..8) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建测试 Provider
        let provider_id = format!("test-provider-cycle-{}", uuid::Uuid::new_v4());
        ctx.create_test_provider(&provider_id).expect("Failed to create provider");

        // 添加 N 个 API Keys
        for i in 0..num_keys {
            let api_key = format!("sk-cycle-key-{provider_id}-{i}");
            ctx.add_test_api_key(&provider_id, &api_key).expect("Failed to add API key");
        }

        // 获取 2N 次 API Key
        let mut first_cycle = Vec::new();
        let mut second_cycle = Vec::new();

        for i in 0..(num_keys * 2) {
            let key = ctx.service
                .get_next_api_key(&ctx.db, &provider_id)
                .expect("Failed to get next API key")
                .expect("No API key returned");

            if i < num_keys {
                first_cycle.push(key);
            } else {
                second_cycle.push(key);
            }
        }

        // 验证：第一轮和第二轮应该返回相同的 Key 序列
        prop_assert_eq!(
            first_cycle,
            second_cycle,
            "Round robin should cycle through keys in the same order"
        );
    }
}

// ============================================================================
// Property 13: API Key 使用统计正确性
// **Validates: Requirements 7.4**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20))]

    /// Property 13: API Key 使用统计正确性
    ///
    /// *对于任意* API Key 使用记录操作，使用次数应正确递增
    ///
    /// **Feature: provider-ui-refactor, Property 13: API Key 使用统计正确性**
    /// **Validates: Requirements 7.4**
    #[test]
    fn test_usage_count_increment(num_usages in 1usize..10) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建测试 Provider
        let provider_id = format!("test-provider-usage-{}", uuid::Uuid::new_v4());
        ctx.create_test_provider(&provider_id).expect("Failed to create provider");

        // 添加 API Key
        let api_key = format!("sk-usage-test-{provider_id}");
        let entry = ctx.add_test_api_key(&provider_id, &api_key)
            .expect("Failed to add API key");

        // 初始使用次数应为 0
        prop_assert_eq!(entry.usage_count, 0, "Initial usage count should be 0");

        // 记录 N 次使用
        for _ in 0..num_usages {
            ctx.service.record_usage(&ctx.db, &entry.id)
                .expect("Failed to record usage");
        }

        // 获取更新后的 API Key
        let conn = ctx.db.lock().expect("Failed to lock db");
        let updated = ApiKeyProviderDao::get_api_key_by_id(&conn, &entry.id)
            .expect("Failed to get API key")
            .expect("API key not found");

        // 验证：使用次数应等于记录次数
        prop_assert_eq!(
            updated.usage_count as usize,
            num_usages,
            "Usage count should equal number of record_usage calls"
        );

        // 验证：最后使用时间应被更新
        prop_assert!(
            updated.last_used_at.is_some(),
            "last_used_at should be set after usage"
        );
    }

    /// Property 13 补充测试：错误次数递增
    ///
    /// *对于任意* API Key 错误记录操作，错误次数应正确递增
    ///
    /// **Feature: provider-ui-refactor, Property 13: API Key 使用统计正确性**
    /// **Validates: Requirements 7.4**
    #[test]
    fn test_error_count_increment(num_errors in 1usize..10) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建测试 Provider
        let provider_id = format!("test-provider-error-{}", uuid::Uuid::new_v4());
        ctx.create_test_provider(&provider_id).expect("Failed to create provider");

        // 添加 API Key
        let api_key = format!("sk-error-test-{provider_id}");
        let entry = ctx.add_test_api_key(&provider_id, &api_key)
            .expect("Failed to add API key");

        // 初始错误次数应为 0
        prop_assert_eq!(entry.error_count, 0, "Initial error count should be 0");

        // 记录 N 次错误
        for _ in 0..num_errors {
            ctx.service.record_error(&ctx.db, &entry.id)
                .expect("Failed to record error");
        }

        // 获取更新后的 API Key
        let conn = ctx.db.lock().expect("Failed to lock db");
        let updated = ApiKeyProviderDao::get_api_key_by_id(&conn, &entry.id)
            .expect("Failed to get API key")
            .expect("API key not found");

        // 验证：错误次数应等于记录次数
        prop_assert_eq!(
            updated.error_count as usize,
            num_errors,
            "Error count should equal number of record_error calls"
        );
    }

    /// Property 13 补充测试：使用和错误统计独立
    ///
    /// *对于任意* API Key，使用次数和错误次数应独立递增
    ///
    /// **Feature: provider-ui-refactor, Property 13: API Key 使用统计正确性**
    /// **Validates: Requirements 7.4**
    #[test]
    fn test_usage_and_error_independent(
        num_usages in 1usize..5,
        num_errors in 1usize..5
    ) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建测试 Provider
        let provider_id = format!("test-provider-mixed-{}", uuid::Uuid::new_v4());
        ctx.create_test_provider(&provider_id).expect("Failed to create provider");

        // 添加 API Key
        let api_key = format!("sk-mixed-test-{provider_id}");
        let entry = ctx.add_test_api_key(&provider_id, &api_key)
            .expect("Failed to add API key");

        // 交替记录使用和错误
        for i in 0..(num_usages + num_errors) {
            if i < num_usages {
                ctx.service.record_usage(&ctx.db, &entry.id)
                    .expect("Failed to record usage");
            }
            if i < num_errors {
                ctx.service.record_error(&ctx.db, &entry.id)
                    .expect("Failed to record error");
            }
        }

        // 获取更新后的 API Key
        let conn = ctx.db.lock().expect("Failed to lock db");
        let updated = ApiKeyProviderDao::get_api_key_by_id(&conn, &entry.id)
            .expect("Failed to get API key")
            .expect("API key not found");

        // 验证：使用次数和错误次数应独立
        prop_assert_eq!(
            updated.usage_count as usize,
            num_usages,
            "Usage count should equal number of record_usage calls"
        );
        prop_assert_eq!(
            updated.error_count as usize,
            num_errors,
            "Error count should equal number of record_error calls"
        );
    }
}

// ============================================================================
// Property 16: 数据持久化 Round-Trip
// **Validates: Requirements 9.1**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 16: 数据持久化 Round-Trip
    ///
    /// *对于任意* Provider 配置，保存后重新加载应得到等价的配置数据
    ///
    /// **Feature: provider-ui-refactor, Property 16: 数据持久化 Round-Trip**
    /// **Validates: Requirements 9.1**
    #[test]
    fn test_provider_persistence_round_trip(
        name in "[a-zA-Z0-9 ]{3,30}",
        api_host in "https://[a-z]{3,10}\\.[a-z]{2,5}/[a-z]{0,10}"
    ) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建 Provider
        let provider = ctx.service
            .add_custom_provider(
                &ctx.db,
                name.clone(),
                ApiProviderType::Openai,
                api_host.clone(),
                None,
                None,
                None,
                None,
            )
            .expect("Failed to create provider");

        // 重新加载 Provider
        let loaded = ctx.service
            .get_provider(&ctx.db, &provider.id)
            .expect("Failed to get provider")
            .expect("Provider not found");

        // 验证：加载的数据应与保存的数据等价
        prop_assert_eq!(&loaded.provider.id, &provider.id, "ID should match");
        prop_assert_eq!(&loaded.provider.name, &name, "Name should match");
        prop_assert_eq!(&loaded.provider.api_host, &api_host, "API host should match");
        prop_assert_eq!(loaded.provider.is_system, false, "Should not be system provider");
        prop_assert_eq!(loaded.provider.group, ProviderGroup::Custom, "Group should be Custom");
    }

    /// Property 16 补充测试：UI 状态持久化 Round-Trip
    ///
    /// *对于任意* UI 状态键值对，保存后重新加载应得到相同的值
    ///
    /// **Feature: provider-ui-refactor, Property 16: 数据持久化 Round-Trip**
    /// **Validates: Requirements 9.1, 8.4**
    #[test]
    fn test_ui_state_persistence_round_trip(
        key in "[a-z_]{3,20}",
        value in "[a-zA-Z0-9_,\\[\\]\"{}:]{1,100}"
    ) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 保存 UI 状态
        ctx.service
            .set_ui_state(&ctx.db, &key, &value)
            .expect("Failed to set UI state");

        // 重新加载 UI 状态
        let loaded = ctx.service
            .get_ui_state(&ctx.db, &key)
            .expect("Failed to get UI state")
            .expect("UI state not found");

        // 验证：加载的值应与保存的值相同
        prop_assert_eq!(&loaded, &value, "UI state value should match");
    }

    /// Property 16 补充测试：Provider 排序持久化 Round-Trip
    ///
    /// *对于任意* Provider 排序顺序，保存后重新加载应保持相同的顺序
    ///
    /// **Feature: provider-ui-refactor, Property 16: 数据持久化 Round-Trip**
    /// **Validates: Requirements 9.1, 8.4**
    #[test]
    fn test_provider_sort_order_persistence(num_providers in 2usize..6) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建多个 Provider
        let mut provider_ids = Vec::new();
        for i in 0..num_providers {
            let provider = ctx.service
                .add_custom_provider(
                    &ctx.db,
                    format!("Provider {i}"),
                    ApiProviderType::Openai,
                    format!("https://api{i}.test.com"),
                    None,
                    None,
                    None,
                    None,
                )
                .expect("Failed to create provider");
            provider_ids.push(provider.id);
        }

        // 反转排序顺序
        let reversed_ids: Vec<_> = provider_ids.iter().rev().cloned().collect();
        let sort_orders: Vec<(String, i32)> = reversed_ids
            .iter()
            .enumerate()
            .map(|(i, id)| (id.clone(), i as i32))
            .collect();

        // 更新排序顺序
        ctx.service
            .update_provider_sort_orders(&ctx.db, sort_orders)
            .expect("Failed to update sort orders");

        // 重新加载所有 Provider
        let loaded = ctx.service
            .get_all_providers(&ctx.db)
            .expect("Failed to get providers");

        // 过滤出我们创建的 Provider
        let our_providers: Vec<_> = loaded
            .iter()
            .filter(|p| provider_ids.contains(&p.provider.id))
            .collect();

        // 验证：排序顺序应与更新后的顺序一致
        for (i, expected_id) in reversed_ids.iter().enumerate() {
            let provider = our_providers
                .iter()
                .find(|p| &p.provider.id == expected_id)
                .expect("Provider not found");
            prop_assert_eq!(
                provider.provider.sort_order,
                i as i32,
                "Sort order should match for provider {}",
                expected_id
            );
        }
    }

    /// Property 16 补充测试：API Key 持久化 Round-Trip
    ///
    /// *对于任意* API Key，保存后重新加载应得到等价的数据（除了加密的 key）
    ///
    /// **Feature: provider-ui-refactor, Property 16: 数据持久化 Round-Trip**
    /// **Validates: Requirements 9.1**
    #[test]
    fn test_api_key_persistence_round_trip(
        api_key in "[a-zA-Z0-9_-]{20,50}",
        alias in proptest::option::of("[a-zA-Z0-9 ]{3,20}")
    ) {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建 Provider
        let provider_id = format!("test-provider-key-rt-{}", uuid::Uuid::new_v4());
        ctx.create_test_provider(&provider_id).expect("Failed to create provider");

        // 添加 API Key
        let entry = ctx.service
            .add_api_key(&ctx.db, &provider_id, &api_key, alias.clone())
            .expect("Failed to add API key");

        // 重新加载 Provider（包含 API Keys）
        let loaded = ctx.service
            .get_provider(&ctx.db, &provider_id)
            .expect("Failed to get provider")
            .expect("Provider not found");

        // 找到我们添加的 API Key
        let loaded_key = loaded.api_keys
            .iter()
            .find(|k| k.id == entry.id)
            .expect("API Key not found");

        // 验证：加载的数据应与保存的数据等价
        prop_assert_eq!(&loaded_key.id, &entry.id, "ID should match");
        prop_assert_eq!(&loaded_key.provider_id, &provider_id, "Provider ID should match");
        prop_assert_eq!(&loaded_key.alias, &alias, "Alias should match");
        prop_assert_eq!(loaded_key.enabled, true, "Should be enabled by default");
        prop_assert_eq!(loaded_key.usage_count, 0, "Usage count should be 0");
        prop_assert_eq!(loaded_key.error_count, 0, "Error count should be 0");

        // 验证：解密后的 API Key 应与原始值相同
        let decrypted = ctx.service
            .decrypt_api_key(&loaded_key.api_key_encrypted)
            .expect("Failed to decrypt");
        prop_assert_eq!(&decrypted, &api_key, "Decrypted API key should match original");
    }
}

// ============================================================================
// Property 17: API Key 加密存储
// **Validates: Requirements 9.2**
// ============================================================================

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// Property 17: API Key 加密存储
    ///
    /// *对于任意* 存储的 API Key，数据库中的值不应为明文
    ///
    /// **Feature: provider-ui-refactor, Property 17: API Key 加密存储**
    /// **Validates: Requirements 9.2**
    #[test]
    fn test_api_key_encryption(api_key in "[a-zA-Z0-9_-]{20,50}") {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建测试 Provider
        let provider_id = format!("test-provider-enc-{}", uuid::Uuid::new_v4());
        ctx.create_test_provider(&provider_id).expect("Failed to create provider");

        // 添加 API Key
        let entry = ctx.add_test_api_key(&provider_id, &api_key)
            .expect("Failed to add API key");

        // 验证：存储的值不是明文
        prop_assert_ne!(
            &entry.api_key_encrypted,
            &api_key,
            "API Key should be encrypted, not stored as plaintext"
        );

        // 验证：加密后的值看起来像 Base64
        prop_assert!(
            entry.api_key_encrypted.chars().all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='),
            "Encrypted value should be Base64 encoded"
        );

        // 验证：可以正确解密
        let decrypted = ctx.service.decrypt_api_key(&entry.api_key_encrypted)
            .expect("Failed to decrypt API key");
        prop_assert_eq!(
            &decrypted,
            &api_key,
            "Decrypted key should match original"
        );
    }

    /// Property 17 补充测试：加密 Round-Trip
    ///
    /// *对于任意* API Key，加密后解密应得到原始值
    ///
    /// **Feature: provider-ui-refactor, Property 17: API Key 加密存储**
    /// **Validates: Requirements 9.2**
    #[test]
    fn test_encryption_round_trip(api_key in "[a-zA-Z0-9_-]{10,100}") {
        let service = ApiKeyProviderService::new();

        // 加密
        let encrypted = service.encrypt_api_key(&api_key);

        // 验证：加密后不等于原文
        prop_assert_ne!(
            &encrypted,
            &api_key,
            "Encrypted value should differ from original"
        );

        // 解密
        let decrypted = service.decrypt_api_key(&encrypted)
            .expect("Failed to decrypt");

        // 验证：解密后等于原文
        prop_assert_eq!(
            &decrypted,
            &api_key,
            "Decrypted value should match original"
        );
    }
}

#[cfg(test)]
mod unit_tests {
    use super::*;

    /// 单元测试：基本的 Provider CRUD 操作
    #[test]
    fn test_provider_crud() {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建 Provider
        let provider = ctx
            .service
            .add_custom_provider(
                &ctx.db,
                "Test Provider".to_string(),
                ApiProviderType::Openai,
                "https://api.test.com".to_string(),
                None,
                None,
                None,
                None,
            )
            .expect("Failed to create provider");

        assert!(provider.id.starts_with("custom-"));
        assert_eq!(provider.name, "Test Provider");
        assert!(!provider.is_system);

        // 获取 Provider
        let retrieved = ctx
            .service
            .get_provider(&ctx.db, &provider.id)
            .expect("Failed to get provider")
            .expect("Provider not found");

        assert_eq!(retrieved.provider.id, provider.id);

        // 更新 Provider
        let updated = ctx
            .service
            .update_provider(
                &ctx.db,
                &provider.id,
                Some("Updated Name".to_string()),
                None,        // provider_type
                None,        // api_host
                Some(false), // enabled
                None,        // sort_order
                None,        // api_version
                None,        // project
                None,        // location
                None,        // region
                None,        // custom_models
            )
            .expect("Failed to update provider");

        assert_eq!(updated.name, "Updated Name");
        assert!(!updated.enabled);

        // 删除 Provider
        let deleted = ctx
            .service
            .delete_custom_provider(&ctx.db, &provider.id)
            .expect("Failed to delete provider");

        assert!(deleted);

        // 验证已删除
        let not_found = ctx
            .service
            .get_provider(&ctx.db, &provider.id)
            .expect("Failed to get provider");

        assert!(not_found.is_none());
    }

    /// 单元测试：API Key CRUD 操作
    #[test]
    fn test_api_key_crud() {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建 Provider
        let provider_id = "test-provider-key-crud";
        ctx.create_test_provider(provider_id)
            .expect("Failed to create provider");

        // 添加 API Key
        let key = ctx
            .add_test_api_key(provider_id, "sk-test-key-123")
            .expect("Failed to add API key");

        assert!(!key.id.is_empty());
        assert_eq!(key.provider_id, provider_id);
        assert!(key.enabled);

        // 切换启用状态
        let toggled = ctx
            .service
            .toggle_api_key(&ctx.db, &key.id, false)
            .expect("Failed to toggle API key");

        assert!(!toggled.enabled);

        // 更新别名
        let aliased = ctx
            .service
            .update_api_key_alias(&ctx.db, &key.id, Some("My Key".to_string()))
            .expect("Failed to update alias");

        assert_eq!(aliased.alias, Some("My Key".to_string()));

        // 删除 API Key
        let deleted = ctx
            .service
            .delete_api_key(&ctx.db, &key.id)
            .expect("Failed to delete API key");

        assert!(deleted);
    }

    /// 单元测试：重复 API Key 检测
    /// 验证修复：第一次添加 API Key 无法保存的问题
    #[test]
    fn test_duplicate_api_key_detection() {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建 Provider
        let provider_id = "test-provider-duplicate";
        ctx.create_test_provider(provider_id)
            .expect("Failed to create provider");

        // 第一次添加 API Key 应该成功
        let api_key = "sk-duplicate-test-123";
        let first_result = ctx.add_test_api_key(provider_id, api_key);
        assert!(first_result.is_ok(), "第一次添加应该成功");

        // 第二次添加相同的 API Key 应该失败
        let second_result = ctx.add_test_api_key(provider_id, api_key);
        assert!(second_result.is_err(), "第二次添加相同 API Key 应该失败");
        assert!(
            second_result.unwrap_err().contains("该 API Key 已存在"),
            "错误信息应该提示 API Key 已存在"
        );

        // 验证 Provider 中只有一个 API Key
        let provider = ctx
            .service
            .get_provider(&ctx.db, provider_id)
            .expect("Failed to get provider")
            .expect("Provider not found");

        assert_eq!(provider.api_keys.len(), 1, "应该只有一个 API Key");
    }

    /// 单元测试：系统 Provider 不能删除
    #[test]
    fn test_system_provider_cannot_be_deleted() {
        let ctx = TestContext::new().expect("Failed to create test context");

        // 创建系统 Provider
        let now = chrono::Utc::now();
        let provider = ApiKeyProvider {
            id: "system-openai".to_string(),
            name: "OpenAI".to_string(),
            provider_type: ApiProviderType::Openai,
            api_host: "https://api.openai.com".to_string(),
            is_system: true, // 系统 Provider
            group: ProviderGroup::Mainstream,
            enabled: true,
            sort_order: 1,
            api_version: None,
            project: None,
            location: None,
            region: None,
            custom_models: vec![],
            created_at: now,
            updated_at: now,
        };

        {
            let conn = ctx.db.lock().expect("Failed to lock db");
            ApiKeyProviderDao::insert_provider(&conn, &provider).expect("Failed to insert");
        }

        // 尝试删除系统 Provider
        let result = ctx.service.delete_custom_provider(&ctx.db, "system-openai");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("不允许删除系统 Provider"));
    }
}
