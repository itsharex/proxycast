//! Workspace Tauri 命令模块
//!
//! 提供 Workspace 管理功能的前端调用接口。
//!
//! ## 主要命令
//! - `workspace_create` - 创建新 workspace
//! - `workspace_list` - 获取 workspace 列表
//! - `workspace_get` - 获取 workspace 详情
//! - `workspace_update` - 更新 workspace
//! - `workspace_delete` - 删除 workspace
//! - `workspace_set_default` - 设置默认 workspace
//! - `workspace_get_default` - 获取默认 workspace

use crate::database::DbConnection;
use crate::workspace::{
    Workspace, WorkspaceManager, WorkspaceSettings, WorkspaceType, WorkspaceUpdate,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;
use tokio::sync::RwLock;

/// Workspace 管理器状态
#[allow(dead_code)]
pub struct WorkspaceManagerState(pub Arc<RwLock<Option<WorkspaceManager>>>);

/// Workspace 列表项（前端展示用）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListItem {
    pub id: String,
    pub name: String,
    pub workspace_type: String,
    pub root_path: String,
    pub is_default: bool,
    pub created_at: i64,
    pub updated_at: i64,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub is_favorite: bool,
    pub is_archived: bool,
    pub tags: Vec<String>,
}

impl From<Workspace> for WorkspaceListItem {
    fn from(ws: Workspace) -> Self {
        Self {
            id: ws.id,
            name: ws.name,
            workspace_type: ws.workspace_type.as_str().to_string(),
            root_path: ws.root_path.to_string_lossy().to_string(),
            is_default: ws.is_default,
            created_at: ws.created_at.timestamp_millis(),
            updated_at: ws.updated_at.timestamp_millis(),
            icon: ws.icon,
            color: ws.color,
            is_favorite: ws.is_favorite,
            is_archived: ws.is_archived,
            tags: ws.tags,
        }
    }
}

/// 创建 workspace 请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkspaceRequest {
    pub name: String,
    pub root_path: String,
    #[serde(default)]
    pub workspace_type: Option<String>,
}

/// 更新 workspace 请求
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkspaceRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub settings: Option<WorkspaceSettings>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub is_favorite: Option<bool>,
    #[serde(default)]
    pub is_archived: Option<bool>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

// ==================== Tauri 命令 ====================

/// 创建新 workspace
#[tauri::command]
pub async fn workspace_create(
    db: State<'_, DbConnection>,
    request: CreateWorkspaceRequest,
) -> Result<WorkspaceListItem, String> {
    let manager = WorkspaceManager::new(db.inner().clone());

    let workspace_type = request
        .workspace_type
        .map(|t| WorkspaceType::from_str(&t))
        .unwrap_or_default();

    let workspace = manager.create_with_type(
        request.name,
        PathBuf::from(&request.root_path),
        workspace_type,
    )?;

    Ok(workspace.into())
}

/// 获取 workspace 列表
#[tauri::command]
pub async fn workspace_list(db: State<'_, DbConnection>) -> Result<Vec<WorkspaceListItem>, String> {
    let manager = WorkspaceManager::new(db.inner().clone());
    let workspaces = manager.list()?;
    Ok(workspaces.into_iter().map(|ws| ws.into()).collect())
}

/// 获取 workspace 详情
#[tauri::command]
pub async fn workspace_get(
    db: State<'_, DbConnection>,
    id: String,
) -> Result<Option<WorkspaceListItem>, String> {
    let manager = WorkspaceManager::new(db.inner().clone());
    let workspace = manager.get(&id)?;
    Ok(workspace.map(|ws| ws.into()))
}

/// 更新 workspace
#[tauri::command]
pub async fn workspace_update(
    db: State<'_, DbConnection>,
    id: String,
    request: UpdateWorkspaceRequest,
) -> Result<WorkspaceListItem, String> {
    let manager = WorkspaceManager::new(db.inner().clone());

    let updates = WorkspaceUpdate {
        name: request.name,
        settings: request.settings,
        icon: request.icon,
        color: request.color,
        is_favorite: request.is_favorite,
        is_archived: request.is_archived,
        tags: request.tags,
    };

    let workspace = manager.update(&id, updates)?;
    Ok(workspace.into())
}

/// 删除 workspace
#[tauri::command]
pub async fn workspace_delete(
    db: State<'_, DbConnection>,
    id: String,
    delete_directory: Option<bool>,
) -> Result<bool, String> {
    let manager = WorkspaceManager::new(db.inner().clone());

    // 如果需要删除目录，先获取 workspace 信息
    if delete_directory.unwrap_or(false) {
        if let Some(workspace) = manager.get(&id)? {
            let root_path = workspace.root_path;
            if root_path.exists() && root_path.is_dir() {
                std::fs::remove_dir_all(&root_path).map_err(|e| format!("删除目录失败: {e}"))?;
                tracing::info!("[Workspace] 删除目录: {:?}", root_path);
            }
        }
    }

    manager.delete(&id)
}

/// 设置默认 workspace
#[tauri::command]
pub async fn workspace_set_default(db: State<'_, DbConnection>, id: String) -> Result<(), String> {
    let manager = WorkspaceManager::new(db.inner().clone());
    manager.set_default(&id)
}

/// 获取默认 workspace
#[tauri::command]
pub async fn workspace_get_default(
    db: State<'_, DbConnection>,
) -> Result<Option<WorkspaceListItem>, String> {
    let manager = WorkspaceManager::new(db.inner().clone());
    let workspace = manager.get_default()?;
    Ok(workspace.map(|ws| ws.into()))
}

/// 通过路径获取 workspace
#[tauri::command]
pub async fn workspace_get_by_path(
    db: State<'_, DbConnection>,
    root_path: String,
) -> Result<Option<WorkspaceListItem>, String> {
    let manager = WorkspaceManager::new(db.inner().clone());
    let workspace = manager.get_by_path(&PathBuf::from(&root_path))?;
    Ok(workspace.map(|ws| ws.into()))
}
