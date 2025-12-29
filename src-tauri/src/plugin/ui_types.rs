//! 插件 UI 类型定义
//!
//! 基于 A2UI 设计理念的声明式 UI 类型系统
//! 插件通过这些类型声明 UI 结构，宿主应用负责渲染

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Surface ID
pub type SurfaceId = String;

/// 组件 ID
pub type ComponentId = String;

/// 数据路径 (JSONPath 格式)
pub type DataPath = String;

// ============================================================================
// 数据绑定
// ============================================================================

/// 绑定值 - 支持字面值或数据路径绑定
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoundValue {
    /// 字符串字面值
    LiteralString { literal_string: String },
    /// 数字字面值
    LiteralNumber { literal_number: f64 },
    /// 布尔字面值
    LiteralBoolean { literal_boolean: bool },
    /// 路径绑定
    Path { path: DataPath },
    /// 字符串 + 路径（初始化）
    StringWithPath {
        literal_string: String,
        path: DataPath,
    },
    /// 数字 + 路径（初始化）
    NumberWithPath { literal_number: f64, path: DataPath },
    /// 布尔 + 路径（初始化）
    BooleanWithPath {
        literal_boolean: bool,
        path: DataPath,
    },
}

impl BoundValue {
    /// 创建字符串字面值
    pub fn string(s: impl Into<String>) -> Self {
        BoundValue::LiteralString {
            literal_string: s.into(),
        }
    }

    /// 创建数字字面值
    pub fn number(n: f64) -> Self {
        BoundValue::LiteralNumber { literal_number: n }
    }

    /// 创建布尔字面值
    pub fn boolean(b: bool) -> Self {
        BoundValue::LiteralBoolean { literal_boolean: b }
    }

    /// 创建路径绑定
    pub fn path(p: impl Into<String>) -> Self {
        BoundValue::Path { path: p.into() }
    }
}

// ============================================================================
// 子组件定义
// ============================================================================

/// 子组件列表定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildrenDef {
    /// 显式列表 - 固定的子组件 ID 列表
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explicit_list: Option<Vec<ComponentId>>,
    /// 模板 - 从数据列表动态生成子组件
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<TemplateDef>,
}

/// 模板定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemplateDef {
    /// 模板组件 ID
    pub component_id: ComponentId,
    /// 数据绑定路径
    pub data_binding: DataPath,
}

impl ChildrenDef {
    /// 创建显式列表
    pub fn explicit(ids: Vec<impl Into<String>>) -> Self {
        Self {
            explicit_list: Some(ids.into_iter().map(|s| s.into()).collect()),
            template: None,
        }
    }

    /// 创建模板
    pub fn template(component_id: impl Into<String>, data_binding: impl Into<String>) -> Self {
        Self {
            explicit_list: None,
            template: Some(TemplateDef {
                component_id: component_id.into(),
                data_binding: data_binding.into(),
            }),
        }
    }
}

// ============================================================================
// 操作定义
// ============================================================================

/// 操作上下文项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionContextItem {
    pub key: String,
    pub value: BoundValue,
}

/// 操作定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<Vec<ActionContextItem>>,
}

impl Action {
    /// 创建简单操作
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            context: None,
        }
    }

    /// 添加上下文
    pub fn with_context(mut self, key: impl Into<String>, value: BoundValue) -> Self {
        let item = ActionContextItem {
            key: key.into(),
            value,
        };
        self.context.get_or_insert_with(Vec::new).push(item);
        self
    }
}

// ============================================================================
// 组件类型
// ============================================================================

/// 文本变体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TextVariant {
    H1,
    H2,
    H3,
    H4,
    H5,
    #[default]
    Body,
    Caption,
}

/// 按钮变体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

/// Badge 变体
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BadgeVariant {
    #[default]
    Default,
    Success,
    Warning,
    Error,
    Info,
}

/// Alert 类型
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AlertType {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// 对齐方式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Alignment {
    Start,
    #[default]
    Center,
    End,
    Stretch,
}

/// 分布方式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub enum Distribution {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// 方向
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Horizontal,
    #[default]
    Vertical,
}

// ============================================================================
// 组件定义
// ============================================================================

/// Row 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowProps {
    pub children: ChildrenDef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution: Option<Distribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<Alignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<u32>,
}

/// Column 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnProps {
    pub children: ChildrenDef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distribution: Option<Distribution>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<Alignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<u32>,
}

/// Card 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardProps {
    pub child: ComponentId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<BoundValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<BoundValue>,
}

/// Text 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextProps {
    pub text: BoundValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<TextVariant>,
}

/// Icon 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconProps {
    pub name: BoundValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
}

/// Button 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ButtonProps {
    pub child: ComponentId,
    pub action: Action,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<ButtonVariant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<BoundValue>,
}

/// Badge 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeProps {
    pub text: BoundValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<BoundValue>,
}

/// Progress 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressProps {
    pub value: BoundValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// TextField 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFieldProps {
    pub label: BoundValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<BoundValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<BoundValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<BoundValue>,
}

/// Switch 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchProps {
    pub label: BoundValue,
    pub checked: BoundValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<BoundValue>,
}

/// List 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProps {
    pub children: ChildrenDef,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alignment: Option<Alignment>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<u32>,
}

/// Alert 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertProps {
    pub message: BoundValue,
    #[serde(rename = "type")]
    pub alert_type: AlertType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<BoundValue>,
}

/// Spinner 组件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpinnerProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<u32>,
}

/// Empty 组件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmptyProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<BoundValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<BoundValue>,
}

/// Divider 组件
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DividerProps {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub axis: Option<String>,
}

/// KeyValue 项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueItem {
    pub key: BoundValue,
    pub value: BoundValue,
}

/// KeyValue 组件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyValueProps {
    pub items: Vec<KeyValueItem>,
}

/// 组件类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ComponentType {
    Row(RowProps),
    Column(ColumnProps),
    Card(CardProps),
    Text(TextProps),
    Icon(IconProps),
    Button(ButtonProps),
    Badge(BadgeProps),
    Progress(ProgressProps),
    TextField(TextFieldProps),
    Switch(SwitchProps),
    List(ListProps),
    Alert(AlertProps),
    Spinner(SpinnerProps),
    Empty(EmptyProps),
    Divider(DividerProps),
    KeyValue(KeyValueProps),
}

/// 组件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDef {
    /// 组件 ID
    pub id: ComponentId,
    /// 组件类型和属性
    pub component: ComponentType,
    /// flex-grow 权重
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
}

impl ComponentDef {
    /// 创建组件定义
    pub fn new(id: impl Into<String>, component: ComponentType) -> Self {
        Self {
            id: id.into(),
            component,
            weight: None,
        }
    }

    /// 设置权重
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}

// ============================================================================
// 消息类型 (Server → Client)
// ============================================================================

/// Surface 更新消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceUpdate {
    pub surface_id: SurfaceId,
    pub components: Vec<ComponentDef>,
}

/// 数据条目
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataEntry {
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_string: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_number: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_boolean: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_array: Option<Vec<DataEntry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_map: Option<Vec<DataEntry>>,
}

impl DataEntry {
    /// 创建字符串条目
    pub fn string(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value_string: Some(value.into()),
            value_number: None,
            value_boolean: None,
            value_array: None,
            value_map: None,
        }
    }

    /// 创建数字条目
    pub fn number(key: impl Into<String>, value: f64) -> Self {
        Self {
            key: key.into(),
            value_string: None,
            value_number: Some(value),
            value_boolean: None,
            value_array: None,
            value_map: None,
        }
    }

    /// 创建布尔条目
    pub fn boolean(key: impl Into<String>, value: bool) -> Self {
        Self {
            key: key.into(),
            value_string: None,
            value_number: None,
            value_boolean: Some(value),
            value_array: None,
            value_map: None,
        }
    }

    /// 创建 Map 条目
    pub fn map(key: impl Into<String>, entries: Vec<DataEntry>) -> Self {
        Self {
            key: key.into(),
            value_string: None,
            value_number: None,
            value_boolean: None,
            value_array: None,
            value_map: Some(entries),
        }
    }
}

/// 数据模型更新消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DataModelUpdate {
    pub surface_id: SurfaceId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<DataPath>,
    pub contents: Vec<DataEntry>,
}

/// Surface 样式
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceStyles {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<u32>,
}

/// 开始渲染消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeginRendering {
    pub surface_id: SurfaceId,
    pub root: ComponentId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub catalog_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<SurfaceStyles>,
}

/// 删除 Surface 消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSurface {
    pub surface_id: SurfaceId,
}

/// 服务端消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UIMessage {
    SurfaceUpdate(SurfaceUpdate),
    DataModelUpdate(DataModelUpdate),
    BeginRendering(BeginRendering),
    DeleteSurface(DeleteSurface),
}

// ============================================================================
// 消息类型 (Client → Server)
// ============================================================================

/// 用户操作消息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserAction {
    pub name: String,
    pub surface_id: SurfaceId,
    pub source_component_id: ComponentId,
    pub context: HashMap<String, serde_json::Value>,
    pub timestamp: String,
}

// ============================================================================
// Surface 定义
// ============================================================================

/// Surface 定义 - 插件返回的初始 UI 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SurfaceDefinition {
    /// Surface ID
    pub surface_id: SurfaceId,
    /// 根组件 ID
    pub root_id: ComponentId,
    /// 初始组件列表
    pub initial_components: Vec<ComponentDef>,
    /// 初始数据
    pub initial_data: serde_json::Value,
    /// 样式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub styles: Option<SurfaceStyles>,
}

impl SurfaceDefinition {
    /// 转换为 UI 消息列表
    pub fn to_messages(&self) -> Vec<UIMessage> {
        let mut messages = Vec::new();

        // 1. Surface 更新
        messages.push(UIMessage::SurfaceUpdate(SurfaceUpdate {
            surface_id: self.surface_id.clone(),
            components: self.initial_components.clone(),
        }));

        // 2. 数据模型更新
        if let Some(obj) = self.initial_data.as_object() {
            let contents: Vec<DataEntry> = obj
                .iter()
                .filter_map(|(k, v)| json_to_data_entry(k, v))
                .collect();

            if !contents.is_empty() {
                messages.push(UIMessage::DataModelUpdate(DataModelUpdate {
                    surface_id: self.surface_id.clone(),
                    path: None,
                    contents,
                }));
            }
        }

        // 3. 开始渲染
        messages.push(UIMessage::BeginRendering(BeginRendering {
            surface_id: self.surface_id.clone(),
            root: self.root_id.clone(),
            catalog_id: None,
            styles: self.styles.clone(),
        }));

        messages
    }
}

/// 将 JSON 值转换为 DataEntry
fn json_to_data_entry(key: &str, value: &serde_json::Value) -> Option<DataEntry> {
    match value {
        serde_json::Value::String(s) => Some(DataEntry::string(key, s)),
        serde_json::Value::Number(n) => n.as_f64().map(|f| DataEntry::number(key, f)),
        serde_json::Value::Bool(b) => Some(DataEntry::boolean(key, *b)),
        serde_json::Value::Object(obj) => {
            let entries: Vec<DataEntry> = obj
                .iter()
                .filter_map(|(k, v)| json_to_data_entry(k, v))
                .collect();
            Some(DataEntry::map(key, entries))
        }
        serde_json::Value::Array(arr) => {
            let entries: Vec<DataEntry> = arr
                .iter()
                .enumerate()
                .filter_map(|(i, v)| json_to_data_entry(&i.to_string(), v))
                .collect();
            Some(DataEntry {
                key: key.to_string(),
                value_string: None,
                value_number: None,
                value_boolean: None,
                value_array: Some(entries),
                value_map: None,
            })
        }
        serde_json::Value::Null => None,
    }
}
