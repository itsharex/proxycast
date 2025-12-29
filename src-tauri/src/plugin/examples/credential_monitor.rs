//! 凭证监控示例插件
//!
//! 演示如何使用插件 UI 系统创建一个凭证状态监控界面

use async_trait::async_trait;
use serde_json::json;

use crate::plugin::ui_trait::PluginUI;
use crate::plugin::ui_types::*;
use crate::plugin::PluginError;

/// 凭证监控插件
pub struct CredentialMonitorPlugin {
    /// 凭证数据
    credentials: Vec<CredentialInfo>,
}

/// 凭证信息
#[derive(Debug, Clone)]
struct CredentialInfo {
    id: String,
    name: String,
    provider: String,
    status: String,
    last_used: Option<String>,
}

impl CredentialMonitorPlugin {
    /// 创建新实例
    pub fn new() -> Self {
        Self {
            credentials: Vec::new(),
        }
    }

    /// 模拟加载凭证数据
    pub fn load_credentials(&mut self) {
        self.credentials = vec![
            CredentialInfo {
                id: "cred-1".into(),
                name: "Kiro Account 1".into(),
                provider: "kiro".into(),
                status: "healthy".into(),
                last_used: Some("2024-01-15 10:30".into()),
            },
            CredentialInfo {
                id: "cred-2".into(),
                name: "Gemini API".into(),
                provider: "gemini".into(),
                status: "healthy".into(),
                last_used: Some("2024-01-15 09:45".into()),
            },
            CredentialInfo {
                id: "cred-3".into(),
                name: "OpenAI Key".into(),
                provider: "openai".into(),
                status: "error".into(),
                last_used: None,
            },
        ];
    }

    /// 构建 UI 组件
    fn build_components(&self) -> Vec<ComponentDef> {
        vec![
            // 根组件 - 垂直布局
            ComponentDef::new(
                "root",
                ComponentType::Column(ColumnProps {
                    children: ChildrenDef::explicit(vec!["header", "divider", "content"]),
                    distribution: Some(Distribution::Start),
                    alignment: Some(Alignment::Stretch),
                    gap: Some(16),
                }),
            ),
            // 头部
            ComponentDef::new(
                "header",
                ComponentType::Row(RowProps {
                    children: ChildrenDef::explicit(vec!["title", "refresh-btn"]),
                    distribution: Some(Distribution::SpaceBetween),
                    alignment: Some(Alignment::Center),
                    gap: Some(8),
                }),
            ),
            // 标题
            ComponentDef::new(
                "title",
                ComponentType::Text(TextProps {
                    text: BoundValue::string("凭证监控"),
                    variant: Some(TextVariant::H3),
                }),
            ),
            // 刷新按钮
            ComponentDef::new(
                "refresh-btn",
                ComponentType::Button(ButtonProps {
                    child: "refresh-btn-content".into(),
                    action: Action::new("refresh"),
                    variant: Some(ButtonVariant::Outline),
                    disabled: None,
                }),
            ),
            // 刷新按钮内容
            ComponentDef::new(
                "refresh-btn-content",
                ComponentType::Row(RowProps {
                    children: ChildrenDef::explicit(vec!["refresh-icon", "refresh-text"]),
                    distribution: Some(Distribution::Center),
                    alignment: Some(Alignment::Center),
                    gap: Some(4),
                }),
            ),
            ComponentDef::icon("refresh-icon", "refresh"),
            ComponentDef::text_literal("refresh-text", "刷新"),
            // 分隔线
            ComponentDef::divider("divider"),
            // 内容区域
            ComponentDef::new(
                "content",
                ComponentType::Column(ColumnProps {
                    children: ChildrenDef::explicit(vec!["stats-row", "credential-list"]),
                    distribution: Some(Distribution::Start),
                    alignment: Some(Alignment::Stretch),
                    gap: Some(16),
                }),
            ),
            // 统计行
            ComponentDef::new(
                "stats-row",
                ComponentType::Row(RowProps {
                    children: ChildrenDef::explicit(vec![
                        "total-card",
                        "healthy-card",
                        "error-card",
                    ]),
                    distribution: Some(Distribution::Start),
                    alignment: Some(Alignment::Stretch),
                    gap: Some(12),
                }),
            ),
            // 统计卡片
            self.build_stat_card("total-card", "total-content", "总数", "/stats/total"),
            self.build_stat_card("healthy-card", "healthy-content", "正常", "/stats/healthy"),
            self.build_stat_card("error-card", "error-content", "异常", "/stats/error"),
            // 凭证列表
            ComponentDef::new(
                "credential-list",
                ComponentType::List(ListProps {
                    children: ChildrenDef::template("credential-item", "/credentials"),
                    direction: Some(Direction::Vertical),
                    alignment: Some(Alignment::Stretch),
                    gap: Some(8),
                }),
            ),
            // 凭证项模板
            self.build_credential_item_template(),
        ]
    }

    /// 构建统计卡片
    fn build_stat_card(
        &self,
        card_id: &str,
        content_id: &str,
        label: &str,
        _value_path: &str,
    ) -> ComponentDef {
        // 这里简化处理，实际应该返回多个组件
        ComponentDef::new(
            card_id,
            ComponentType::Card(CardProps {
                child: content_id.into(),
                title: Some(BoundValue::string(label)),
                description: None,
            }),
        )
    }

    /// 构建凭证项模板
    fn build_credential_item_template(&self) -> ComponentDef {
        ComponentDef::new(
            "credential-item",
            ComponentType::Card(CardProps {
                child: "item-row".into(),
                title: None,
                description: None,
            }),
        )
    }

    /// 构建初始数据
    fn build_data(&self) -> serde_json::Value {
        let credentials: Vec<serde_json::Value> = self
            .credentials
            .iter()
            .map(|c| {
                json!({
                    "id": c.id,
                    "name": c.name,
                    "provider": c.provider,
                    "status": c.status,
                    "statusVariant": if c.status == "healthy" { "success" } else { "error" },
                    "lastUsed": c.last_used.clone().unwrap_or_else(|| "从未使用".into())
                })
            })
            .collect();

        let healthy_count = self
            .credentials
            .iter()
            .filter(|c| c.status == "healthy")
            .count();
        let error_count = self.credentials.len() - healthy_count;

        json!({
            "stats": {
                "total": self.credentials.len(),
                "healthy": healthy_count,
                "error": error_count
            },
            "credentials": credentials
        })
    }
}

impl Default for CredentialMonitorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl PluginUI for CredentialMonitorPlugin {
    fn get_surfaces(&self) -> Vec<SurfaceDefinition> {
        vec![SurfaceDefinition {
            surface_id: "credential-monitor".into(),
            root_id: "root".into(),
            initial_components: self.build_components(),
            initial_data: self.build_data(),
            styles: Some(SurfaceStyles {
                primary_color: Some("#3b82f6".into()),
                font: None,
                border_radius: Some(8),
            }),
        }]
    }

    async fn handle_action(&mut self, action: UserAction) -> Result<Vec<UIMessage>, PluginError> {
        match action.name.as_str() {
            "refresh" => {
                // 重新加载凭证数据
                self.load_credentials();

                // 返回数据更新消息
                Ok(vec![UIMessage::DataModelUpdate(DataModelUpdate {
                    surface_id: "credential-monitor".into(),
                    path: None,
                    contents: vec![DataEntry::map(
                        "stats",
                        vec![
                            DataEntry::number("total", self.credentials.len() as f64),
                            DataEntry::number(
                                "healthy",
                                self.credentials
                                    .iter()
                                    .filter(|c| c.status == "healthy")
                                    .count() as f64,
                            ),
                            DataEntry::number(
                                "error",
                                self.credentials
                                    .iter()
                                    .filter(|c| c.status != "healthy")
                                    .count() as f64,
                            ),
                        ],
                    )],
                })])
            }
            _ => {
                tracing::debug!("未知操作: {}", action.name);
                Ok(Vec::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_surfaces() {
        let mut plugin = CredentialMonitorPlugin::new();
        plugin.load_credentials();

        let surfaces = plugin.get_surfaces();
        assert_eq!(surfaces.len(), 1);
        assert_eq!(surfaces[0].surface_id, "credential-monitor");
        assert!(!surfaces[0].initial_components.is_empty());
    }

    #[tokio::test]
    async fn test_refresh_action() {
        let mut plugin = CredentialMonitorPlugin::new();
        plugin.load_credentials();

        let action = UserAction {
            name: "refresh".into(),
            surface_id: "credential-monitor".into(),
            source_component_id: "refresh-btn".into(),
            context: std::collections::HashMap::new(),
            timestamp: "2024-01-15T10:00:00Z".into(),
        };

        let messages = plugin.handle_action(action).await.unwrap();
        assert!(!messages.is_empty());
    }
}
