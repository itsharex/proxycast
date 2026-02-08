//! MCP 客户端实现
//!
//! 实现 rmcp 的 ClientHandler trait，处理通知和回调。
//! 使用 DynEmitter 替代 Tauri AppHandle 进行事件发射。

#![allow(dead_code)]

use proxycast_core::DynEmitter;
use rmcp::{
    model::{
        ClientCapabilities, ClientInfo, Implementation, LoggingMessageNotification,
        LoggingMessageNotificationMethod, LoggingMessageNotificationParam, ProgressNotification,
        ProgressNotificationMethod, ProgressNotificationParam, ProtocolVersion, ServerNotification,
    },
    service::NotificationContext,
    ClientHandler, RoleClient,
};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, info, warn};

/// 进度通知事件 Payload
#[derive(Debug, Clone, serde::Serialize)]
pub struct McpProgressPayload {
    pub server_name: String,
    pub progress_token: String,
    pub progress: f64,
    pub total: Option<f64>,
    pub message: Option<String>,
}

/// 日志消息事件 Payload
#[derive(Debug, Clone, serde::Serialize)]
pub struct McpLogMessagePayload {
    pub server_name: String,
    pub level: String,
    pub logger: Option<String>,
    pub data: serde_json::Value,
}

/// ProxyCast MCP 客户端处理器
pub struct ProxyCastMcpClient {
    emitter: Option<DynEmitter>,
    server_name: String,
    notification_handlers: Arc<Mutex<Vec<mpsc::Sender<ServerNotification>>>>,
}

impl ProxyCastMcpClient {
    pub fn new(server_name: String, emitter: Option<DynEmitter>) -> Self {
        Self {
            emitter,
            server_name,
            notification_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn notification_handlers(&self) -> Arc<Mutex<Vec<mpsc::Sender<ServerNotification>>>> {
        self.notification_handlers.clone()
    }

    pub async fn subscribe(&self) -> mpsc::Receiver<ServerNotification> {
        let (tx, rx) = mpsc::channel(16);
        self.notification_handlers.lock().await.push(tx);
        rx
    }

    /// 发送事件（通过 DynEmitter）
    fn emit_event<T: serde::Serialize>(&self, event: &str, payload: &T) {
        if let Some(ref emitter) = self.emitter {
            if let Ok(value) = serde_json::to_value(payload) {
                if let Err(e) = emitter.emit_event(event, &value) {
                    warn!(
                        server_name = %self.server_name,
                        event = %event,
                        error = %e,
                        "发送事件失败"
                    );
                }
            }
        }
    }
}

impl ClientHandler for ProxyCastMcpClient {
    fn get_info(&self) -> ClientInfo {
        ClientInfo {
            protocol_version: ProtocolVersion::V_2025_03_26,
            capabilities: ClientCapabilities::builder().enable_sampling().build(),
            client_info: Implementation {
                name: "proxycast".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("ProxyCast MCP Client".to_string()),
                website_url: Some("https://github.com/aiclientproxy/proxycast".to_string()),
            },
        }
    }

    async fn on_progress(
        &self,
        params: ProgressNotificationParam,
        context: NotificationContext<RoleClient>,
    ) {
        debug!(
            server_name = %self.server_name,
            progress_token = ?params.progress_token,
            progress = params.progress,
            total = ?params.total,
            "收到 MCP 进度通知"
        );

        let payload = McpProgressPayload {
            server_name: self.server_name.clone(),
            progress_token: format!("{:?}", params.progress_token),
            progress: params.progress,
            total: params.total,
            message: None,
        };
        self.emit_event("mcp:progress", &payload);

        let notification = ServerNotification::ProgressNotification(ProgressNotification {
            params: params.clone(),
            method: ProgressNotificationMethod,
            extensions: context.extensions.clone(),
        });

        let handlers = self.notification_handlers.lock().await;
        for handler in handlers.iter() {
            let _ = handler.try_send(notification.clone());
        }
    }

    async fn on_logging_message(
        &self,
        params: LoggingMessageNotificationParam,
        context: NotificationContext<RoleClient>,
    ) {
        let level_str = format!("{:?}", params.level);
        match params.level {
            rmcp::model::LoggingLevel::Debug => {
                debug!(server_name = %self.server_name, logger = ?params.logger, data = ?params.data, "MCP 服务器日志 [DEBUG]");
            }
            rmcp::model::LoggingLevel::Info => {
                info!(server_name = %self.server_name, logger = ?params.logger, data = ?params.data, "MCP 服务器日志 [INFO]");
            }
            rmcp::model::LoggingLevel::Notice => {
                info!(server_name = %self.server_name, logger = ?params.logger, data = ?params.data, "MCP 服务器日志 [NOTICE]");
            }
            rmcp::model::LoggingLevel::Warning => {
                warn!(server_name = %self.server_name, logger = ?params.logger, data = ?params.data, "MCP 服务器日志 [WARNING]");
            }
            _ => {
                tracing::error!(server_name = %self.server_name, logger = ?params.logger, data = ?params.data, level = %level_str, "MCP 服务器日志");
            }
        }

        let payload = McpLogMessagePayload {
            server_name: self.server_name.clone(),
            level: level_str,
            logger: params.logger.clone(),
            data: params.data.clone(),
        };
        self.emit_event("mcp:log_message", &payload);

        let notification =
            ServerNotification::LoggingMessageNotification(LoggingMessageNotification {
                params: params.clone(),
                method: LoggingMessageNotificationMethod,
                extensions: context.extensions.clone(),
            });

        let handlers = self.notification_handlers.lock().await;
        for handler in handlers.iter() {
            let _ = handler.try_send(notification.clone());
        }
    }
}

/// MCP 客户端包装器
pub struct McpClientWrapper {
    pub server_name: String,
    pub config: super::types::McpServerConfig,
    pub process: Option<tokio::process::Child>,
    pub server_info: Option<super::types::McpServerCapabilities>,
    pub client_handler: Arc<ProxyCastMcpClient>,
    pub running_service:
        Option<rmcp::service::RunningService<rmcp::RoleClient, ProxyCastMcpClient>>,
}

impl McpClientWrapper {
    pub fn new(
        server_name: String,
        config: super::types::McpServerConfig,
        emitter: Option<DynEmitter>,
    ) -> Self {
        let client_handler = Arc::new(ProxyCastMcpClient::new(server_name.clone(), emitter));

        Self {
            server_name,
            config,
            process: None,
            server_info: None,
            client_handler,
            running_service: None,
        }
    }

    pub fn handler(&self) -> Arc<ProxyCastMcpClient> {
        self.client_handler.clone()
    }

    pub fn set_process(&mut self, process: tokio::process::Child) {
        self.process = Some(process);
    }

    pub fn set_server_info(&mut self, info: super::types::McpServerCapabilities) {
        self.server_info = Some(info);
    }

    pub fn set_running_service(
        &mut self,
        service: rmcp::service::RunningService<rmcp::RoleClient, ProxyCastMcpClient>,
    ) {
        self.running_service = Some(service);
    }

    pub fn running_service(
        &self,
    ) -> Option<&rmcp::service::RunningService<rmcp::RoleClient, ProxyCastMcpClient>> {
        self.running_service.as_ref()
    }

    pub async fn kill_process(&mut self) -> Result<(), std::io::Error> {
        if let Some(ref mut process) = self.process {
            process.kill().await?;
        }
        self.process = None;
        self.running_service = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_info() {
        let client = ProxyCastMcpClient::new("test-server".to_string(), None);
        let info = client.get_info();

        assert_eq!(info.client_info.name, "proxycast");
        assert_eq!(
            info.client_info.title,
            Some("ProxyCast MCP Client".to_string())
        );
        assert_eq!(info.protocol_version, ProtocolVersion::V_2025_03_26);
    }

    #[test]
    fn test_client_wrapper_creation() {
        let config = super::super::types::McpServerConfig {
            command: "test-command".to_string(),
            args: vec!["--arg1".to_string()],
            env: std::collections::HashMap::new(),
            cwd: None,
            timeout: 30,
        };

        let wrapper = McpClientWrapper::new("test-server".to_string(), config, None);

        assert_eq!(wrapper.server_name, "test-server");
        assert_eq!(wrapper.config.command, "test-command");
        assert!(wrapper.process.is_none());
        assert!(wrapper.server_info.is_none());
    }

    #[tokio::test]
    async fn test_notification_subscription() {
        let client = ProxyCastMcpClient::new("test-server".to_string(), None);

        let mut rx = client.subscribe().await;

        let handlers = client.notification_handlers.lock().await;
        assert_eq!(handlers.len(), 1);
        drop(handlers);

        assert!(rx.try_recv().is_err());
    }
}
