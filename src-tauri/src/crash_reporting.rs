//! 崩溃上报初始化与上报辅助（Sentry 协议兼容）

use proxycast_core::config::{Config, CrashReportingConfig};
use serde_json::Value;

/// 根据配置初始化 Sentry 客户端
pub fn init_from_config(config: &Config) -> Option<sentry::ClientInitGuard> {
    init(&config.crash_reporting)
}

fn init(config: &CrashReportingConfig) -> Option<sentry::ClientInitGuard> {
    if !config.enabled {
        tracing::info!("[CrashReporting] 已禁用崩溃上报");
        return None;
    }

    let dsn = config
        .dsn
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())?;

    let sample_rate = config.sample_rate.clamp(0.0, 1.0) as f32;
    let environment = config.environment.trim();
    let environment = if environment.is_empty() {
        "production"
    } else {
        environment
    };

    let guard = sentry::init((
        dsn,
        sentry::ClientOptions {
            release: sentry::release_name!(),
            environment: Some(environment.to_string().into()),
            sample_rate,
            send_default_pii: config.send_pii,
            attach_stacktrace: true,
            ..Default::default()
        },
    ));

    sentry::configure_scope(|scope| {
        scope.set_tag("platform", std::env::consts::OS);
        scope.set_tag("arch", std::env::consts::ARCH);
        scope.set_tag("app", "proxycast");
        scope.set_tag("app_version", env!("CARGO_PKG_VERSION"));
    });

    tracing::info!(
        "[CrashReporting] Sentry 已初始化: env={}, sample_rate={}",
        environment,
        sample_rate
    );

    Some(guard)
}

/// 记录来自前端的崩溃/异常事件（在已初始化时发送到 Sentry）
pub fn capture_frontend_report(
    message: &str,
    component: Option<&str>,
    workflow_step: Option<&str>,
    creation_mode: Option<&str>,
    metadata: Option<Value>,
) {
    if message.trim().is_empty() {
        return;
    }

    sentry::with_scope(
        |scope| {
            scope.set_tag("origin", "frontend");
            if let Some(component_name) = component.filter(|value| !value.trim().is_empty()) {
                scope.set_tag("component", component_name.to_string());
            }
            if let Some(step) = workflow_step.filter(|value| !value.trim().is_empty()) {
                scope.set_tag("workflow_step", step.to_string());
            }
            if let Some(mode) = creation_mode.filter(|value| !value.trim().is_empty()) {
                scope.set_tag("creation_mode", mode.to_string());
            }
            if let Some(extra) = metadata {
                scope.set_extra("frontend_report", sentry::protocol::Value::from(extra));
            }
        },
        || {
            sentry::capture_message(message, sentry::Level::Error);
        },
    );
}
