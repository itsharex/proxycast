//! 内容创作类型定义
//!
//! 定义工作流、步骤、表单等核心数据结构

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 创作主题类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ThemeType {
    /// 通用对话
    General,
    /// 知识探索
    Knowledge,
    /// 计划制定
    Planning,
    /// 社媒内容
    SocialMedia,
    /// 海报设计
    Poster,
    /// 文档写作
    Document,
    /// 论文写作
    Paper,
    /// 小说创作
    Novel,
    /// 剧本创作
    Script,
    /// 音乐创作
    Music,
    /// 视频脚本
    Video,
}

impl Default for ThemeType {
    fn default() -> Self {
        Self::General
    }
}

/// 创作模式
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CreationMode {
    /// 引导模式：AI 提问，用户回答
    Guided,
    /// 快速模式：AI 直接生成
    Fast,
    /// 混合模式：AI 生成框架，用户填核心
    Hybrid,
    /// 框架模式：用户提供框架，AI 填充
    Framework,
}

impl Default for CreationMode {
    fn default() -> Self {
        Self::Guided
    }
}

/// 步骤类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    /// 明确需求
    Clarify,
    /// 调研收集
    Research,
    /// 生成大纲
    Outline,
    /// 撰写内容
    Write,
    /// 润色优化
    Polish,
    /// 适配发布
    Adapt,
}

/// 步骤状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// 待处理
    Pending,
    /// 进行中
    Active,
    /// 已完成
    Completed,
    /// 已跳过
    Skipped,
    /// 错误
    Error,
}

impl Default for StepStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// 表单字段类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum FormFieldType {
    Text,
    Textarea,
    Select,
    Radio,
    Checkbox,
    Slider,
    Tags,
    Outline,
}

/// 表单字段选项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormFieldOption {
    pub label: String,
    pub value: String,
}

/// 表单字段定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: FormFieldType,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<FormFieldOption>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_value: Option<serde_json::Value>,
}

/// 表单配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormConfig {
    pub fields: Vec<FormField>,
    pub submit_label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_label: Option<String>,
}

/// AI 任务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AITaskConfig {
    pub task_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default)]
    pub streaming: bool,
}

/// 步骤行为配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepBehavior {
    /// 是否可跳过
    pub skippable: bool,
    /// 是否可重做
    pub redoable: bool,
    /// 完成后是否自动进入下一步
    pub auto_advance: bool,
}

impl Default for StepBehavior {
    fn default() -> Self {
        Self {
            skippable: false,
            redoable: true,
            auto_advance: true,
        }
    }
}

/// 步骤定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepDefinition {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: StepType,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub form: Option<FormConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_task: Option<AITaskConfig>,
    #[serde(default)]
    pub behavior: StepBehavior,
}

/// 内容文件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub file_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// 步骤结果
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepResult {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_input: Option<HashMap<String, serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ai_output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<ContentFile>>,
}

/// 工作流步骤（运行时状态）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    #[serde(flatten)]
    pub definition: StepDefinition,
    #[serde(default)]
    pub status: StepStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<StepResult>,
}

/// 工作流状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    pub id: String,
    pub content_id: String,
    pub theme: ThemeType,
    pub mode: CreationMode,
    pub steps: Vec<WorkflowStep>,
    pub current_step_index: usize,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 工作流进度（持久化用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub workflow_id: String,
    pub content_id: String,
    pub theme: ThemeType,
    pub mode: CreationMode,
    pub steps_json: String,
    pub current_step_index: i32,
    pub created_at: i64,
    pub updated_at: i64,
}
