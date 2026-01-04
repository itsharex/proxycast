//! ProxyCast Plugin SDK
//!
//! 提供给 OAuth Provider 插件使用的 SDK 接口。
//! 插件可以通过这些接口访问 ProxyCast 的核心功能。

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// SDK 错误类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SdkError {
    /// 数据库错误
    DatabaseError(String),
    /// HTTP 错误
    HttpError(String),
    /// 加密错误
    CryptoError(String),
    /// 权限错误
    PermissionDenied(String),
    /// 未找到
    NotFound(String),
    /// 参数错误
    InvalidArgument(String),
    /// 内部错误
    InternalError(String),
}

impl std::fmt::Display for SdkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SdkError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            SdkError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            SdkError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            SdkError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            SdkError::NotFound(msg) => write!(f, "Not found: {}", msg),
            SdkError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            SdkError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for SdkError {}

/// SDK 结果类型
pub type SdkResult<T> = Result<T, SdkError>;

/// HTTP 请求选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HttpRequestOptions {
    /// HTTP 方法
    #[serde(default = "default_method")]
    pub method: String,
    /// 请求头
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// 请求体
    #[serde(default)]
    pub body: Option<String>,
    /// 超时（毫秒）
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}

fn default_method() -> String {
    "GET".to_string()
}

fn default_timeout() -> u64 {
    30000
}

/// HTTP 响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// 状态码
    pub status: u16,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 响应体
    pub body: String,
}

/// 数据库查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    /// 列名
    pub columns: Vec<String>,
    /// 行数据
    pub rows: Vec<Vec<serde_json::Value>>,
}

/// 插件权限
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PluginPermission {
    /// 读取数据库
    DatabaseRead,
    /// 写入数据库
    DatabaseWrite,
    /// 发送 HTTP 请求
    HttpRequest,
    /// 加密数据
    CryptoEncrypt,
    /// 解密数据
    CryptoDecrypt,
    /// 发送通知
    Notification,
    /// 发布事件
    EventEmit,
    /// 订阅事件
    EventSubscribe,
    /// 访问文件系统
    FileSystemRead,
    /// 写入文件系统
    FileSystemWrite,
}

/// 数据库连接包装
///
/// 由于 rusqlite::Connection 不是 Send + Sync，我们使用回调模式
pub type DatabaseCallback =
    Box<dyn Fn(&str, Vec<serde_json::Value>) -> Result<QueryResult, String> + Send + Sync>;

/// 插件 SDK 上下文
///
/// 提供给插件的 SDK 接口，包含所有可用的功能。
pub struct PluginSdkContext {
    /// 插件 ID
    pub plugin_id: String,
    /// 授予的权限
    pub permissions: Vec<PluginPermission>,
    /// 数据库查询回调
    db_query_callback: Option<Arc<DatabaseCallback>>,
    /// HTTP 客户端
    http_client: reqwest::Client,
}

impl PluginSdkContext {
    /// 创建新的 SDK 上下文
    pub fn new(plugin_id: String, permissions: Vec<PluginPermission>) -> Self {
        Self {
            plugin_id,
            permissions,
            db_query_callback: None,
            http_client: reqwest::Client::new(),
        }
    }

    /// 设置数据库查询回调
    pub fn with_database_callback(mut self, callback: DatabaseCallback) -> Self {
        self.db_query_callback = Some(Arc::new(callback));
        self
    }

    /// 检查权限
    fn check_permission(&self, required: PluginPermission) -> SdkResult<()> {
        if self.permissions.contains(&required) {
            Ok(())
        } else {
            Err(SdkError::PermissionDenied(format!(
                "Plugin '{}' does not have {:?} permission",
                self.plugin_id, required
            )))
        }
    }

    // ========================================================================
    // 数据库操作
    // ========================================================================

    /// 执行数据库查询
    pub async fn database_query(
        &self,
        sql: &str,
        params: Vec<serde_json::Value>,
    ) -> SdkResult<QueryResult> {
        self.check_permission(PluginPermission::DatabaseRead)?;

        let callback = self
            .db_query_callback
            .as_ref()
            .ok_or_else(|| SdkError::DatabaseError("Database not initialized".to_string()))?;

        // 安全检查：只允许 SELECT 语句
        let sql_upper = sql.trim().to_uppercase();
        if !sql_upper.starts_with("SELECT") {
            return Err(SdkError::PermissionDenied(
                "Only SELECT queries are allowed for database_query".to_string(),
            ));
        }

        // 限制只能查询插件自己的表或公共表
        if !self.is_allowed_table(sql) {
            return Err(SdkError::PermissionDenied(
                "Access to this table is not allowed".to_string(),
            ));
        }

        // 执行数据库查询
        callback(sql, params).map_err(|e| SdkError::DatabaseError(e))
    }

    /// 执行数据库写入
    pub async fn database_execute(
        &self,
        sql: &str,
        _params: Vec<serde_json::Value>,
    ) -> SdkResult<u64> {
        self.check_permission(PluginPermission::DatabaseWrite)?;

        let _callback = self
            .db_query_callback
            .as_ref()
            .ok_or_else(|| SdkError::DatabaseError("Database not initialized".to_string()))?;

        // 限制只能操作插件自己的表
        if !self.is_plugin_table(sql) {
            return Err(SdkError::PermissionDenied(
                "Can only modify plugin-owned tables".to_string(),
            ));
        }

        // TODO: 执行实际的数据库写入
        Ok(0)
    }

    /// 检查是否是允许访问的表
    fn is_allowed_table(&self, sql: &str) -> bool {
        let sql_lower = sql.to_lowercase();

        // 允许访问的公共表
        let public_tables = ["credential_provider_plugins", "plugin_credentials"];

        // 检查是否访问公共表
        for table in public_tables {
            if sql_lower.contains(table) {
                return true;
            }
        }

        // 检查是否访问插件自己的表（以 plugin_{plugin_id}_ 为前缀）
        let plugin_prefix = format!("plugin_{}.", self.plugin_id.replace('-', "_"));
        sql_lower.contains(&plugin_prefix)
    }

    /// 检查是否是插件自己的表
    fn is_plugin_table(&self, sql: &str) -> bool {
        let sql_lower = sql.to_lowercase();
        let plugin_prefix = format!("plugin_{}.", self.plugin_id.replace('-', "_"));
        sql_lower.contains(&plugin_prefix)
    }

    // ========================================================================
    // HTTP 操作
    // ========================================================================

    /// 发送 HTTP 请求
    pub async fn http_request(
        &self,
        url: &str,
        options: HttpRequestOptions,
    ) -> SdkResult<HttpResponse> {
        self.check_permission(PluginPermission::HttpRequest)?;

        let method = options.method.to_uppercase();
        let mut request = match method.as_str() {
            "GET" => self.http_client.get(url),
            "POST" => self.http_client.post(url),
            "PUT" => self.http_client.put(url),
            "DELETE" => self.http_client.delete(url),
            "PATCH" => self.http_client.patch(url),
            "HEAD" => self.http_client.head(url),
            _ => {
                return Err(SdkError::InvalidArgument(format!(
                    "Unsupported HTTP method: {}",
                    method
                )))
            }
        };

        // 添加请求头
        for (key, value) in options.headers {
            request = request.header(&key, &value);
        }

        // 添加请求体
        if let Some(body) = options.body {
            request = request.body(body);
        }

        // 设置超时
        request = request.timeout(std::time::Duration::from_millis(options.timeout_ms));

        // 发送请求
        let response = request
            .send()
            .await
            .map_err(|e| SdkError::HttpError(e.to_string()))?;

        let status = response.status().as_u16();
        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| {
                v.to_str()
                    .ok()
                    .map(|v| (k.as_str().to_string(), v.to_string()))
            })
            .collect();

        let body = response
            .text()
            .await
            .map_err(|e| SdkError::HttpError(e.to_string()))?;

        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }

    // ========================================================================
    // 加密操作
    // ========================================================================

    /// 加密数据
    pub async fn crypto_encrypt(&self, data: &str) -> SdkResult<String> {
        self.check_permission(PluginPermission::CryptoEncrypt)?;

        // TODO: 使用 ProxyCast 的加密服务
        // 暂时使用 base64 编码作为占位符
        use base64::Engine;
        Ok(base64::engine::general_purpose::STANDARD.encode(data.as_bytes()))
    }

    /// 解密数据
    pub async fn crypto_decrypt(&self, data: &str) -> SdkResult<String> {
        self.check_permission(PluginPermission::CryptoDecrypt)?;

        // TODO: 使用 ProxyCast 的解密服务
        // 暂时使用 base64 解码作为占位符
        use base64::Engine;
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(data)
            .map_err(|e| SdkError::CryptoError(e.to_string()))?;

        String::from_utf8(bytes).map_err(|e| SdkError::CryptoError(e.to_string()))
    }

    // ========================================================================
    // 通知操作
    // ========================================================================

    /// 发送成功通知
    pub fn notification_success(&self, message: &str) -> SdkResult<()> {
        self.check_permission(PluginPermission::Notification)?;
        tracing::info!("[Plugin {}] Success: {}", self.plugin_id, message);
        // TODO: 发送到前端通知系统
        Ok(())
    }

    /// 发送错误通知
    pub fn notification_error(&self, message: &str) -> SdkResult<()> {
        self.check_permission(PluginPermission::Notification)?;
        tracing::error!("[Plugin {}] Error: {}", self.plugin_id, message);
        // TODO: 发送到前端通知系统
        Ok(())
    }

    /// 发送信息通知
    pub fn notification_info(&self, message: &str) -> SdkResult<()> {
        self.check_permission(PluginPermission::Notification)?;
        tracing::info!("[Plugin {}] Info: {}", self.plugin_id, message);
        // TODO: 发送到前端通知系统
        Ok(())
    }

    // ========================================================================
    // 事件操作
    // ========================================================================

    /// 发布事件
    pub fn event_emit(&self, event: &str, data: serde_json::Value) -> SdkResult<()> {
        self.check_permission(PluginPermission::EventEmit)?;
        tracing::debug!(
            "[Plugin {}] Emitting event '{}': {:?}",
            self.plugin_id,
            event,
            data
        );
        // TODO: 通过事件总线发布事件
        Ok(())
    }

    // ========================================================================
    // 插件存储
    // ========================================================================

    /// 获取插件存储的值
    pub async fn storage_get(&self, _key: &str) -> SdkResult<Option<String>> {
        self.check_permission(PluginPermission::DatabaseRead)?;

        // TODO: 从插件存储表读取
        Ok(None)
    }

    /// 设置插件存储的值
    pub async fn storage_set(&self, _key: &str, _value: &str) -> SdkResult<()> {
        self.check_permission(PluginPermission::DatabaseWrite)?;

        // TODO: 写入插件存储表
        Ok(())
    }

    /// 删除插件存储的值
    pub async fn storage_delete(&self, _key: &str) -> SdkResult<()> {
        self.check_permission(PluginPermission::DatabaseWrite)?;

        // TODO: 从插件存储表删除
        Ok(())
    }
}

/// JSON-RPC 请求格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC 版本
    pub jsonrpc: String,
    /// 方法名
    pub method: String,
    /// 参数
    pub params: serde_json::Value,
    /// 请求 ID
    pub id: serde_json::Value,
}

/// JSON-RPC 响应格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC 版本
    pub jsonrpc: String,
    /// 结果（成功时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// 错误（失败时）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    /// 请求 ID
    pub id: serde_json::Value,
}

/// JSON-RPC 错误
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// 错误码
    pub code: i32,
    /// 错误消息
    pub message: String,
    /// 附加数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

impl JsonRpcResponse {
    /// 创建成功响应
    pub fn success(id: serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    /// 创建错误响应
    pub fn error(id: serde_json::Value, code: i32, message: String) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code,
                message,
                data: None,
            }),
            id,
        }
    }
}

/// SDK 方法处理器
///
/// 处理来自外部插件的 SDK 调用请求
pub struct SdkMethodHandler {
    context: PluginSdkContext,
}

impl SdkMethodHandler {
    /// 创建新的处理器
    pub fn new(context: PluginSdkContext) -> Self {
        Self { context }
    }

    /// 处理 JSON-RPC 请求
    pub async fn handle(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            // 数据库方法
            "database.query" => self.handle_database_query(request).await,
            "database.execute" => self.handle_database_execute(request).await,

            // HTTP 方法
            "http.request" => self.handle_http_request(request).await,

            // 加密方法
            "crypto.encrypt" => self.handle_crypto_encrypt(request).await,
            "crypto.decrypt" => self.handle_crypto_decrypt(request).await,

            // 通知方法
            "notification.success" => self.handle_notification(request, "success"),
            "notification.error" => self.handle_notification(request, "error"),
            "notification.info" => self.handle_notification(request, "info"),

            // 事件方法
            "event.emit" => self.handle_event_emit(request),

            // 存储方法
            "storage.get" => self.handle_storage_get(request).await,
            "storage.set" => self.handle_storage_set(request).await,
            "storage.delete" => self.handle_storage_delete(request).await,

            // 未知方法
            _ => JsonRpcResponse::error(
                request.id,
                -32601,
                format!("Method not found: {}", request.method),
            ),
        }
    }

    async fn handle_database_query(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            sql: String,
            #[serde(default)]
            params: Vec<serde_json::Value>,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self
                .context
                .database_query(&params.sql, params.params)
                .await
            {
                Ok(result) => {
                    JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
                }
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    async fn handle_database_execute(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            sql: String,
            #[serde(default)]
            params: Vec<serde_json::Value>,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => {
                match self
                    .context
                    .database_execute(&params.sql, params.params)
                    .await
                {
                    Ok(affected) => JsonRpcResponse::success(
                        request.id,
                        serde_json::json!({ "affected": affected }),
                    ),
                    Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
                }
            }
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    async fn handle_http_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            url: String,
            #[serde(default)]
            options: HttpRequestOptions,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self.context.http_request(&params.url, params.options).await {
                Ok(response) => {
                    JsonRpcResponse::success(request.id, serde_json::to_value(response).unwrap())
                }
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    async fn handle_crypto_encrypt(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            data: String,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self.context.crypto_encrypt(&params.data).await {
                Ok(encrypted) => JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({ "encrypted": encrypted }),
                ),
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    async fn handle_crypto_decrypt(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            data: String,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self.context.crypto_decrypt(&params.data).await {
                Ok(decrypted) => JsonRpcResponse::success(
                    request.id,
                    serde_json::json!({ "decrypted": decrypted }),
                ),
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    fn handle_notification(&self, request: JsonRpcRequest, level: &str) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            message: String,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => {
                let result = match level {
                    "success" => self.context.notification_success(&params.message),
                    "error" => self.context.notification_error(&params.message),
                    "info" => self.context.notification_info(&params.message),
                    _ => Ok(()),
                };
                match result {
                    Ok(()) => JsonRpcResponse::success(request.id, serde_json::json!({})),
                    Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
                }
            }
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    fn handle_event_emit(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            event: String,
            data: serde_json::Value,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self.context.event_emit(&params.event, params.data) {
                Ok(()) => JsonRpcResponse::success(request.id, serde_json::json!({})),
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    async fn handle_storage_get(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            key: String,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self.context.storage_get(&params.key).await {
                Ok(value) => {
                    JsonRpcResponse::success(request.id, serde_json::json!({ "value": value }))
                }
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    async fn handle_storage_set(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            key: String,
            value: String,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self.context.storage_set(&params.key, &params.value).await {
                Ok(()) => JsonRpcResponse::success(request.id, serde_json::json!({})),
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }

    async fn handle_storage_delete(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        #[derive(Deserialize)]
        struct Params {
            key: String,
        }

        match serde_json::from_value::<Params>(request.params.clone()) {
            Ok(params) => match self.context.storage_delete(&params.key).await {
                Ok(()) => JsonRpcResponse::success(request.id, serde_json::json!({})),
                Err(e) => JsonRpcResponse::error(request.id, -32000, e.to_string()),
            },
            Err(e) => JsonRpcResponse::error(request.id, -32602, format!("Invalid params: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sdk_context_permission_check() {
        let context = PluginSdkContext::new(
            "test-plugin".to_string(),
            vec![PluginPermission::DatabaseRead],
        );

        assert!(context
            .check_permission(PluginPermission::DatabaseRead)
            .is_ok());
        assert!(context
            .check_permission(PluginPermission::DatabaseWrite)
            .is_err());
    }

    #[test]
    fn test_json_rpc_response() {
        let success =
            JsonRpcResponse::success(serde_json::json!(1), serde_json::json!({"result": "ok"}));
        assert!(success.result.is_some());
        assert!(success.error.is_none());

        let error = JsonRpcResponse::error(serde_json::json!(1), -32000, "Error".to_string());
        assert!(error.result.is_none());
        assert!(error.error.is_some());
    }

    #[test]
    fn test_is_allowed_table() {
        let context = PluginSdkContext::new("kiro-provider".to_string(), vec![]);

        // 公共表应该允许
        assert!(context.is_allowed_table("SELECT * FROM credential_provider_plugins"));
        assert!(context.is_allowed_table("SELECT * FROM plugin_credentials"));

        // 插件自己的表应该允许
        assert!(context.is_allowed_table("SELECT * FROM plugin_kiro_provider.accounts"));

        // 其他表应该禁止
        assert!(!context.is_allowed_table("SELECT * FROM api_keys"));
        assert!(!context.is_allowed_table("SELECT * FROM plugin_other.data"));
    }
}
