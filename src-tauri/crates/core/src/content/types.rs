//! Content 类型定义
//!
//! 定义项目内容相关的数据结构和类型。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Content 唯一标识
pub type ContentId = String;

/// 内容类型
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    /// 剧集（短剧项目）
    Episode,
    /// 章节（小说项目）
    Chapter,
    /// 帖子（社媒项目）
    Post,
    /// 文档（文档项目）
    #[default]
    Document,
    /// 通用内容
    Content,
}

impl ContentType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Episode => "episode",
            ContentType::Chapter => "chapter",
            ContentType::Post => "post",
            ContentType::Document => "document",
            ContentType::Content => "content",
        }
    }

    /// 获取内容类型的显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            ContentType::Episode => "剧集",
            ContentType::Chapter => "章节",
            ContentType::Post => "帖子",
            ContentType::Document => "文档",
            ContentType::Content => "内容",
        }
    }
}

impl std::str::FromStr for ContentType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "episode" => Ok(ContentType::Episode),
            "chapter" => Ok(ContentType::Chapter),
            "post" => Ok(ContentType::Post),
            "document" => Ok(ContentType::Document),
            "content" => Ok(ContentType::Content),
            _ => Ok(ContentType::Document),
        }
    }
}

/// 内容状态
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ContentStatus {
    /// 草稿
    #[default]
    Draft,
    /// 已完成
    Completed,
    /// 已发布
    Published,
}

#[allow(dead_code)]
impl ContentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ContentStatus::Draft => "draft",
            ContentStatus::Completed => "completed",
            ContentStatus::Published => "published",
        }
    }
}

impl std::str::FromStr for ContentStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(ContentStatus::Draft),
            "completed" => Ok(ContentStatus::Completed),
            "published" => Ok(ContentStatus::Published),
            _ => Ok(ContentStatus::Draft),
        }
    }
}

/// 内容实体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    /// 唯一标识
    pub id: ContentId,
    /// 所属项目 ID
    pub project_id: String,
    /// 标题
    pub title: String,
    /// 内容类型
    pub content_type: ContentType,
    /// 状态
    pub status: ContentStatus,
    /// 排序顺序
    pub order: i32,
    /// 正文内容
    pub body: String,
    /// 字数统计
    pub word_count: i64,
    /// 类型特定的元数据（JSON）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// 关联的 AI 会话 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 更新时间
    pub updated_at: DateTime<Utc>,
}

/// 内容创建请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCreateRequest {
    /// 所属项目 ID
    pub project_id: String,
    /// 标题
    pub title: String,
    /// 内容类型（可选，默认根据项目类型推断）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    /// 排序顺序（可选，默认追加到末尾）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
    /// 初始正文内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

/// 内容更新请求
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentUpdateRequest {
    /// 新标题
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// 新状态
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ContentStatus>,
    /// 新排序顺序
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<i32>,
    /// 新正文内容
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// 新元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// 关联的 AI 会话 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

/// 内容列表查询参数
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ContentListQuery {
    /// 按状态过滤
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ContentStatus>,
    /// 按内容类型过滤
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<ContentType>,
    /// 搜索关键词
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search: Option<String>,
    /// 排序字段
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    /// 排序方向（asc/desc）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
    /// 分页：偏移量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// 分页：限制数量
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}

/// 内容版本（用于版本历史）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentVersion {
    /// 版本 ID
    pub id: String,
    /// 内容 ID
    pub content_id: ContentId,
    /// 版本号
    pub version: i32,
    /// 正文内容
    pub body: String,
    /// 字数统计
    pub word_count: i64,
    /// 创建时间
    pub created_at: DateTime<Utc>,
    /// 备注
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}
