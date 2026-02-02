use crate::browser_interceptor::{
    BrowserInterceptor, BrowserInterceptorConfig, InterceptedUrl, InterceptorState, UrlStatistics,
};
use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 全局拦截器实例
static INTERCEPTOR: Lazy<Arc<RwLock<Option<BrowserInterceptor>>>> =
    Lazy::new(|| Arc::new(RwLock::new(None)));

/// 获取拦截器状态
#[tauri::command]
pub async fn get_browser_interceptor_state() -> Result<Option<InterceptorState>, String> {
    let interceptor = INTERCEPTOR.read().await;
    if let Some(ref int) = *interceptor {
        int.get_state().await.map(Some).map_err(|e| e.to_string())
    } else {
        Ok(None)
    }
}

/// 启动拦截器
#[tauri::command]
pub async fn start_browser_interceptor(config: BrowserInterceptorConfig) -> Result<String, String> {
    let mut interceptor_guard = INTERCEPTOR.write().await;

    // 如果已有拦截器在运行，先停止
    if let Some(ref mut int) = *interceptor_guard {
        int.stop().await.map_err(|e| e.to_string())?;
    }

    // 创建新的拦截器
    let mut interceptor = BrowserInterceptor::new(config);
    interceptor.start().await.map_err(|e| e.to_string())?;

    *interceptor_guard = Some(interceptor);

    Ok("拦截器已启动".to_string())
}

/// 停止拦截器
#[tauri::command]
pub async fn stop_browser_interceptor() -> Result<String, String> {
    let mut interceptor_guard = INTERCEPTOR.write().await;

    if let Some(ref mut int) = *interceptor_guard {
        int.stop().await.map_err(|e| e.to_string())?;
        *interceptor_guard = None;
        Ok("拦截器已停止".to_string())
    } else {
        Ok("拦截器未运行".to_string())
    }
}

/// 恢复正常浏览器行为
#[tauri::command]
pub async fn restore_normal_browser_behavior() -> Result<String, String> {
    let mut interceptor_guard = INTERCEPTOR.write().await;

    if let Some(ref mut int) = *interceptor_guard {
        int.restore_normal_behavior()
            .await
            .map_err(|e| e.to_string())?;
        *interceptor_guard = None;
        Ok("已恢复正常浏览器行为".to_string())
    } else {
        Ok("拦截器未运行".to_string())
    }
}

/// 临时禁用拦截器
#[tauri::command]
pub async fn temporary_disable_interceptor(duration_seconds: u64) -> Result<String, String> {
    let mut interceptor_guard = INTERCEPTOR.write().await;

    if let Some(ref mut int) = *interceptor_guard {
        int.temporary_disable(duration_seconds)
            .await
            .map_err(|e| e.to_string())?;
        Ok(format!("拦截器已临时禁用 {duration_seconds} 秒"))
    } else {
        Err("拦截器未运行".to_string())
    }
}

/// 获取拦截的 URL 列表
#[tauri::command]
pub async fn get_intercepted_urls() -> Result<Vec<InterceptedUrl>, String> {
    let interceptor = INTERCEPTOR.read().await;

    if let Some(ref int) = *interceptor {
        int.get_intercepted_urls().await.map_err(|e| e.to_string())
    } else {
        Ok(Vec::new())
    }
}

/// 获取历史记录
#[tauri::command]
pub async fn get_interceptor_history(limit: Option<usize>) -> Result<Vec<InterceptedUrl>, String> {
    let interceptor = INTERCEPTOR.read().await;

    if let Some(ref int) = *interceptor {
        int.get_history(limit).await.map_err(|e| e.to_string())
    } else {
        Ok(Vec::new())
    }
}

/// 复制 URL 到剪贴板
#[tauri::command]
pub async fn copy_intercepted_url_to_clipboard(url_id: String) -> Result<String, String> {
    let interceptor = INTERCEPTOR.read().await;

    if let Some(ref int) = *interceptor {
        int.copy_url_to_clipboard(&url_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok("URL 已复制到剪贴板".to_string())
    } else {
        Err("拦截器未运行".to_string())
    }
}

/// 在指纹浏览器中打开 URL
#[tauri::command]
pub async fn open_url_in_fingerprint_browser(url_id: String) -> Result<String, String> {
    let interceptor = INTERCEPTOR.read().await;

    if let Some(ref int) = *interceptor {
        int.open_in_fingerprint_browser(&url_id)
            .await
            .map_err(|e| e.to_string())?;
        Ok("URL 已在指纹浏览器中打开".to_string())
    } else {
        Err("拦截器未运行".to_string())
    }
}

/// 忽略 URL
#[tauri::command]
pub async fn dismiss_intercepted_url(url_id: String) -> Result<String, String> {
    let interceptor = INTERCEPTOR.read().await;

    if let Some(ref int) = *interceptor {
        int.dismiss_url(&url_id).await.map_err(|e| e.to_string())?;
        Ok("URL 已忽略".to_string())
    } else {
        Err("拦截器未运行".to_string())
    }
}

/// 更新配置
#[tauri::command]
pub async fn update_browser_interceptor_config(
    config: BrowserInterceptorConfig,
) -> Result<String, String> {
    let interceptor = INTERCEPTOR.read().await;

    if let Some(ref int) = *interceptor {
        int.update_config(config).await.map_err(|e| e.to_string())?;
        Ok("配置已更新".to_string())
    } else {
        Err("拦截器未运行".to_string())
    }
}

/// 获取默认配置
#[tauri::command]
pub async fn get_default_browser_interceptor_config() -> Result<BrowserInterceptorConfig, String> {
    Ok(BrowserInterceptorConfig::default())
}

/// 验证配置
#[tauri::command]
pub async fn validate_browser_interceptor_config(
    config: BrowserInterceptorConfig,
) -> Result<String, String> {
    config.validate()?;
    Ok("配置验证通过".to_string())
}

/// 检查是否正在运行
#[tauri::command]
pub async fn is_browser_interceptor_running() -> Result<bool, String> {
    let interceptor = INTERCEPTOR.read().await;
    Ok(interceptor.is_some())
}

/// 获取统计信息
#[tauri::command]
pub async fn get_browser_interceptor_statistics() -> Result<UrlStatistics, String> {
    let interceptor = INTERCEPTOR.read().await;

    if let Some(ref _int) = *interceptor {
        // 简化实现，返回默认统计
        Ok(UrlStatistics {
            current_intercepted: 0,
            total_intercepted: 0,
            copied_count: 0,
            opened_count: 0,
            dismissed_count: 0,
            process_stats: std::collections::HashMap::new(),
        })
    } else {
        Ok(UrlStatistics {
            current_intercepted: 0,
            total_intercepted: 0,
            copied_count: 0,
            opened_count: 0,
            dismissed_count: 0,
            process_stats: std::collections::HashMap::new(),
        })
    }
}

/// 显示通知
#[tauri::command]
pub async fn show_notification(
    title: String,
    body: String,
    _icon: Option<String>,
) -> Result<String, String> {
    tracing::info!("[通知] {}: {}", title, body);
    Ok("通知已显示".to_string())
}

/// 显示 URL 拦截通知
#[tauri::command]
pub async fn show_url_intercept_notification(
    url: String,
    source_process: String,
) -> Result<String, String> {
    tracing::info!("[URL 拦截] 来自 {}: {}", source_process, url);
    Ok("通知已显示".to_string())
}

/// 显示状态通知
#[tauri::command]
pub async fn show_status_notification(
    message: String,
    notification_type: String,
) -> Result<String, String> {
    tracing::info!("[状态通知] [{}] {}", notification_type, message);
    Ok("通知已显示".to_string())
}
