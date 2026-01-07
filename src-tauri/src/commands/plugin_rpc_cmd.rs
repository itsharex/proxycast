//! 插件 RPC 通信命令
//!
//! 提供插件与其 Binary 后端进程的 JSON-RPC 通信功能：
//! - plugin_rpc_connect: 启动插件进程并建立连接
//! - plugin_rpc_disconnect: 关闭插件进程
//! - plugin_rpc_call: 发送 RPC 请求并等待响应
//!
//! 支持异步通知：后端进程可以发送 JSON-RPC 通知，通过 Tauri 事件转发到前端。
//!
//! _需求: 插件 RPC 通信_

use crate::commands::plugin_install_cmd::PluginInstallerState;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::Emitter;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};

/// RPC 请求 ID 生成器
static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

/// JSON-RPC 请求
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    method: String,
    params: Option<Value>,
    id: u64,
}

/// JSON-RPC 响应
#[derive(Debug, Deserialize, Clone)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    result: Option<Value>,
    error: Option<JsonRpcError>,
    id: Option<u64>,
}

/// JSON-RPC 通知（无 id 字段）
#[derive(Debug, Deserialize, Clone)]
struct JsonRpcNotification {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
    params: Option<Value>,
}

/// JSON-RPC 消息（可能是响应或通知）
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum JsonRpcMessage {
    Response(JsonRpcResponse),
    Notification(JsonRpcNotification),
}

/// JSON-RPC 错误
#[derive(Debug, Deserialize, Clone)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[allow(dead_code)]
    data: Option<Value>,
}

/// RPC 通知事件 payload
#[derive(Debug, Clone, Serialize)]
struct RpcNotificationPayload {
    plugin_id: String,
    method: String,
    params: Option<Value>,
}

/// 待处理的 RPC 请求
struct PendingRequest {
    response_tx: oneshot::Sender<Result<Value, String>>,
}

/// 插件进程信息
struct PluginProcess {
    #[allow(dead_code)]
    child: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

/// 插件 RPC 管理器状态
pub struct PluginRpcManagerState {
    /// 运行中的插件进程
    processes: RwLock<HashMap<String, Arc<Mutex<PluginProcess>>>>,
}

impl PluginRpcManagerState {
    pub fn new() -> Self {
        Self {
            processes: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for PluginRpcManagerState {
    fn default() -> Self {
        Self::new()
    }
}

/// 启动插件进程并建立 RPC 连接
#[tauri::command]
pub async fn plugin_rpc_connect(
    plugin_id: String,
    app_handle: tauri::AppHandle,
    installer_state: tauri::State<'_, PluginInstallerState>,
    rpc_state: tauri::State<'_, PluginRpcManagerState>,
) -> Result<(), String> {
    // 检查是否已连接
    {
        let processes = rpc_state.processes.read().await;
        if processes.contains_key(&plugin_id) {
            return Ok(()); // 已连接
        }
    }

    // 获取插件信息
    let installer = installer_state.0.read().await;
    let plugins = installer.list_installed().map_err(|e| e.to_string())?;
    let plugin = plugins
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| format!("插件 {} 未安装", plugin_id))?;

    // 读取插件 manifest
    let manifest_path = plugin.install_path.join("plugin.json");
    let manifest_content = std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("读取 manifest 失败: {}", e))?;
    let manifest: Value = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("解析 manifest 失败: {}", e))?;

    // 获取二进制文件路径
    let _binary_name = manifest["binary"]["binary_name"]
        .as_str()
        .ok_or("manifest 中缺少 binary.binary_name")?;

    // 根据平台选择二进制文件
    let platform_key = match (std::env::consts::ARCH, std::env::consts::OS) {
        ("aarch64", "macos") => "macos-arm64",
        ("x86_64", "macos") => "macos-x64",
        ("x86_64", "linux") => "linux-x64",
        ("aarch64", "linux") => "linux-arm64",
        ("x86_64", "windows") => "windows-x64",
        _ => return Err("不支持的平台".to_string()),
    };

    let binary_filename = manifest["binary"]["platform_binaries"][platform_key]
        .as_str()
        .ok_or_else(|| format!("manifest 中缺少 {} 平台的二进制文件", platform_key))?;

    let binary_path = plugin.install_path.join(binary_filename);
    if !binary_path.exists() {
        return Err(format!("二进制文件不存在: {:?}", binary_path));
    }

    // 启动进程（使用 tokio::process::Command）
    let mut child = Command::new(&binary_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("启动插件进程失败: {}", e))?;

    tracing::info!("插件 {} 进程已启动, PID: {:?}", plugin_id, child.id());

    // 获取 stdin 和 stdout
    let stdin = child.stdin.take().ok_or("无法获取进程 stdin")?;
    let stdout = child.stdout.take().ok_or("无法获取进程 stdout")?;

    let stdin = Arc::new(Mutex::new(stdin));
    let pending_requests: Arc<Mutex<HashMap<u64, PendingRequest>>> =
        Arc::new(Mutex::new(HashMap::new()));

    // 创建 shutdown channel
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

    // 启动 stdout 读取任务
    let plugin_id_clone = plugin_id.clone();
    let pending_requests_clone = pending_requests.clone();
    let app_handle_clone = app_handle.clone();

    tokio::spawn(async move {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();

            tokio::select! {
                // 检查 shutdown 信号
                _ = shutdown_rx.recv() => {
                    tracing::info!("插件 {} stdout 读取任务收到关闭信号", plugin_id_clone);
                    break;
                }
                // 读取一行
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            // EOF
                            tracing::info!("插件 {} stdout EOF", plugin_id_clone);
                            break;
                        }
                        Ok(_) => {
                            let line_trimmed = line.trim();
                            if line_trimmed.is_empty() {
                                continue;
                            }

                            // 尝试解析为 JSON-RPC 消息
                            match serde_json::from_str::<JsonRpcMessage>(line_trimmed) {
                                Ok(JsonRpcMessage::Response(response)) => {
                                    // 这是一个响应，找到对应的 pending request
                                    if let Some(id) = response.id {
                                        let mut pending = pending_requests_clone.lock().await;
                                        if let Some(req) = pending.remove(&id) {
                                            let result = if let Some(error) = response.error {
                                                Err(format!("RPC 错误 [{}]: {}", error.code, error.message))
                                            } else {
                                                Ok(response.result.unwrap_or(Value::Null))
                                            };
                                            let _ = req.response_tx.send(result);
                                        }
                                    }
                                }
                                Ok(JsonRpcMessage::Notification(notification)) => {
                                    // 这是一个通知，发送 Tauri 事件
                                    let payload = RpcNotificationPayload {
                                        plugin_id: plugin_id_clone.clone(),
                                        method: notification.method.clone(),
                                        params: notification.params,
                                    };
                                    tracing::debug!(
                                        "插件 {} 发送通知: {}",
                                        plugin_id_clone,
                                        notification.method
                                    );
                                    if let Err(e) = app_handle_clone.emit("plugin-rpc-notification", &payload) {
                                        tracing::error!("发送 Tauri 事件失败: {}", e);
                                    }
                                }
                                Err(e) => {
                                    tracing::warn!(
                                        "插件 {} 收到无效 JSON: {} - {}",
                                        plugin_id_clone,
                                        e,
                                        line_trimmed
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("插件 {} 读取 stdout 失败: {}", plugin_id_clone, e);
                            break;
                        }
                    }
                }
            }
        }
    });

    let process = PluginProcess {
        child,
        stdin,
        pending_requests,
        shutdown_tx: Some(shutdown_tx),
    };

    // 保存进程
    let mut processes = rpc_state.processes.write().await;
    processes.insert(plugin_id, Arc::new(Mutex::new(process)));

    Ok(())
}

/// 关闭插件 RPC 连接
#[tauri::command]
pub async fn plugin_rpc_disconnect(
    plugin_id: String,
    rpc_state: tauri::State<'_, PluginRpcManagerState>,
) -> Result<(), String> {
    let mut processes = rpc_state.processes.write().await;

    if let Some(process_arc) = processes.remove(&plugin_id) {
        let mut process = process_arc.lock().await;

        // 发送 shutdown 信号
        if let Some(tx) = process.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // 终止进程
        if let Err(e) = process.child.kill().await {
            tracing::warn!("关闭插件 {} 进程失败: {}", plugin_id, e);
        }
        tracing::info!("插件 {} 进程已关闭", plugin_id);
    }

    Ok(())
}

/// 发送 RPC 请求
#[tauri::command]
pub async fn plugin_rpc_call(
    plugin_id: String,
    method: String,
    params: Option<Value>,
    rpc_state: tauri::State<'_, PluginRpcManagerState>,
) -> Result<Value, String> {
    let processes = rpc_state.processes.read().await;
    let process_arc = processes
        .get(&plugin_id)
        .ok_or_else(|| format!("插件 {} 未连接", plugin_id))?
        .clone();
    drop(processes);

    let process = process_arc.lock().await;

    // 构建请求
    let request_id = REQUEST_ID.fetch_add(1, Ordering::SeqCst);
    let request = JsonRpcRequest {
        jsonrpc: "2.0",
        method: method.clone(),
        params,
        id: request_id,
    };

    let request_json =
        serde_json::to_string(&request).map_err(|e| format!("序列化请求失败: {}", e))?;

    // 创建响应 channel
    let (response_tx, response_rx) = oneshot::channel();

    // 注册 pending request
    {
        let mut pending = process.pending_requests.lock().await;
        pending.insert(request_id, PendingRequest { response_tx });
    }

    // 发送请求
    {
        let mut stdin = process.stdin.lock().await;
        stdin
            .write_all(request_json.as_bytes())
            .await
            .map_err(|e| format!("发送请求失败: {}", e))?;
        stdin
            .write_all(b"\n")
            .await
            .map_err(|e| format!("发送换行失败: {}", e))?;
        stdin
            .flush()
            .await
            .map_err(|e| format!("刷新 stdin 失败: {}", e))?;
    }

    // 释放 process lock，让 stdout 读取任务可以处理响应
    drop(process);

    // 等待响应（带超时）
    match tokio::time::timeout(std::time::Duration::from_secs(30), response_rx).await {
        Ok(Ok(result)) => result,
        Ok(Err(_)) => Err("响应 channel 已关闭".to_string()),
        Err(_) => {
            // 超时，清理 pending request
            let process = process_arc.lock().await;
            let mut pending = process.pending_requests.lock().await;
            pending.remove(&request_id);
            Err(format!("RPC 调用 {} 超时", method))
        }
    }
}
