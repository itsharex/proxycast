//! BlockController trait 定义
//!
//! 定义统一的块控制器接口，所有控制器类型（Shell、Cmd、SSH、WSL）都必须实现此 trait。
//!
//! ## 功能
//! - 定义 BlockController trait 接口
//! - 定义 BlockControllerRuntimeStatus 运行时状态结构
//! - 定义 BlockInputUnion 输入联合类型
//!
//! ## Requirements
//! - 1.1: 定义统一的 trait 接口
//! - 1.8: 提供 get_runtime_status 方法

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::terminal::TerminalError;

/// 终端大小
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TermSize {
    /// 行数
    pub rows: u16,
    /// 列数
    pub cols: u16,
}

impl Default for TermSize {
    fn default() -> Self {
        Self { rows: 24, cols: 80 }
    }
}

/// 块控制器运行时状态
///
/// 包含控制器的当前状态信息，用于前端显示和状态同步。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockControllerRuntimeStatus {
    /// 块 ID
    pub block_id: String,
    /// 状态版本号，每次状态变更递增
    pub version: i32,
    /// Shell 进程状态: "init" | "running" | "done"
    pub shell_proc_status: String,
    /// Shell 进程连接名称（用于 SSH/WSL）
    pub shell_proc_conn_name: Option<String>,
    /// Shell 进程退出码
    pub shell_proc_exit_code: i32,
}

impl BlockControllerRuntimeStatus {
    /// 创建初始状态
    pub fn new(block_id: String) -> Self {
        Self {
            block_id,
            version: 0,
            shell_proc_status: "init".to_string(),
            shell_proc_conn_name: None,
            shell_proc_exit_code: 0,
        }
    }

    /// 检查是否为初始状态
    pub fn is_init(&self) -> bool {
        self.shell_proc_status == "init"
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.shell_proc_status == "running"
    }

    /// 检查是否已完成
    pub fn is_done(&self) -> bool {
        self.shell_proc_status == "done"
    }
}

/// 块控制器输入联合类型
///
/// 封装发送给控制器的各种输入类型。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockInputUnion {
    /// 输入数据（键盘输入等）
    pub input_data: Option<Vec<u8>>,
    /// 信号名称（如 "SIGINT", "SIGTERM"）
    pub sig_name: Option<String>,
    /// 终端大小调整
    pub term_size: Option<TermSize>,
}

impl BlockInputUnion {
    /// 创建数据输入
    pub fn data(data: Vec<u8>) -> Self {
        Self {
            input_data: Some(data),
            sig_name: None,
            term_size: None,
        }
    }

    /// 创建信号输入
    pub fn signal(sig_name: &str) -> Self {
        Self {
            input_data: None,
            sig_name: Some(sig_name.to_string()),
            term_size: None,
        }
    }

    /// 创建终端大小调整输入
    pub fn resize(rows: u16, cols: u16) -> Self {
        Self {
            input_data: None,
            sig_name: None,
            term_size: Some(TermSize { rows, cols }),
        }
    }
}

/// 块元数据
///
/// 存储块的配置信息。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockMeta {
    /// 控制器类型: "shell" | "cmd"
    pub controller: Option<String>,
    /// 连接名称（用于 SSH/WSL）
    pub connection: Option<String>,
    /// 命令字符串
    pub cmd: Option<String>,
    /// 命令参数
    pub cmd_args: Option<Vec<String>>,
    /// 工作目录
    pub cmd_cwd: Option<String>,
    /// 环境变量
    pub cmd_env: Option<HashMap<String, String>>,
    /// 启动时自动运行
    pub cmd_run_on_start: Option<bool>,
    /// 仅运行一次
    pub cmd_run_once: Option<bool>,
    /// 启动前清空输出
    pub cmd_clear_on_start: Option<bool>,
    /// 退出后自动关闭
    pub cmd_close_on_exit: Option<bool>,
    /// 终端模式: "term" | "vdom"
    pub term_mode: Option<String>,
    /// 终端主题
    pub term_theme: Option<String>,
    /// 终端字体大小
    pub term_font_size: Option<f32>,
    /// 终端滚动缓冲区大小
    pub term_scrollback: Option<i32>,
}

impl BlockMeta {
    /// 获取字符串字段
    pub fn get_string(&self, key: &str) -> String {
        match key {
            "controller" => self.controller.clone().unwrap_or_default(),
            "connection" => self.connection.clone().unwrap_or_default(),
            "cmd" => self.cmd.clone().unwrap_or_default(),
            "cmd_cwd" => self.cmd_cwd.clone().unwrap_or_default(),
            "term_mode" => self.term_mode.clone().unwrap_or_else(|| "term".to_string()),
            "term_theme" => self.term_theme.clone().unwrap_or_default(),
            _ => String::new(),
        }
    }
}

/// 运行时选项
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeOpts {
    /// 终端大小
    pub term_size: TermSize,
}

/// 块控制器 trait
///
/// 所有控制器类型（Shell、Cmd、SSH、WSL）都必须实现此 trait。
/// 提供统一的接口用于管理终端会话的生命周期。
#[async_trait]
pub trait BlockController: Send + Sync {
    /// 启动控制器
    ///
    /// # 参数
    /// - `block_meta`: 块元数据配置
    /// - `rt_opts`: 运行时选项（终端大小等）
    /// - `force`: 是否强制重启
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误
    async fn start(
        &mut self,
        block_meta: BlockMeta,
        rt_opts: Option<RuntimeOpts>,
        force: bool,
    ) -> Result<(), TerminalError>;

    /// 停止控制器
    ///
    /// # 参数
    /// - `graceful`: 是否优雅停止（发送 SIGTERM 而非 SIGKILL）
    /// - `new_status`: 停止后的新状态
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误
    async fn stop(&mut self, graceful: bool, new_status: String) -> Result<(), TerminalError>;

    /// 获取运行时状态
    ///
    /// # 返回
    /// 当前控制器的运行时状态
    fn get_runtime_status(&self) -> BlockControllerRuntimeStatus;

    /// 发送输入到控制器
    ///
    /// # 参数
    /// - `input`: 输入数据（键盘输入、信号、终端大小调整）
    ///
    /// # 返回
    /// 成功返回 Ok(()), 失败返回错误
    async fn send_input(&self, input: &BlockInputUnion) -> Result<(), TerminalError>;

    /// 获取控制器类型
    ///
    /// # 返回
    /// 控制器类型字符串: "shell" | "cmd"
    fn controller_type(&self) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_size_default() {
        let size = TermSize::default();
        assert_eq!(size.rows, 24);
        assert_eq!(size.cols, 80);
    }

    #[test]
    fn test_runtime_status_new() {
        let status = BlockControllerRuntimeStatus::new("test-block".to_string());
        assert_eq!(status.block_id, "test-block");
        assert_eq!(status.version, 0);
        assert_eq!(status.shell_proc_status, "init");
        assert!(status.is_init());
        assert!(!status.is_running());
        assert!(!status.is_done());
    }

    #[test]
    fn test_block_input_union_data() {
        let input = BlockInputUnion::data(vec![0x1b, 0x5b, 0x41]); // ESC [ A
        assert!(input.input_data.is_some());
        assert!(input.sig_name.is_none());
        assert!(input.term_size.is_none());
    }

    #[test]
    fn test_block_input_union_signal() {
        let input = BlockInputUnion::signal("SIGINT");
        assert!(input.input_data.is_none());
        assert_eq!(input.sig_name, Some("SIGINT".to_string()));
        assert!(input.term_size.is_none());
    }

    #[test]
    fn test_block_input_union_resize() {
        let input = BlockInputUnion::resize(30, 100);
        assert!(input.input_data.is_none());
        assert!(input.sig_name.is_none());
        assert_eq!(
            input.term_size,
            Some(TermSize {
                rows: 30,
                cols: 100
            })
        );
    }

    #[test]
    fn test_block_meta_get_string() {
        let meta = BlockMeta {
            controller: Some("shell".to_string()),
            connection: Some("ssh://user@host".to_string()),
            ..Default::default()
        };
        assert_eq!(meta.get_string("controller"), "shell");
        assert_eq!(meta.get_string("connection"), "ssh://user@host");
        assert_eq!(meta.get_string("cmd"), "");
        assert_eq!(meta.get_string("term_mode"), "term");
    }
}
