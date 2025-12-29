//! 插件 UI 构建器
//!
//! 提供便捷的 API 来构建插件 UI

use super::ui_types::*;
use serde_json::json;

/// Surface 构建器
pub struct SurfaceBuilder {
    surface_id: String,
    root_id: String,
    components: Vec<ComponentDef>,
    data: serde_json::Value,
    styles: Option<SurfaceStyles>,
}

impl SurfaceBuilder {
    /// 创建新的 Surface 构建器
    pub fn new(surface_id: impl Into<String>, root_id: impl Into<String>) -> Self {
        Self {
            surface_id: surface_id.into(),
            root_id: root_id.into(),
            components: Vec::new(),
            data: json!({}),
            styles: None,
        }
    }

    /// 添加组件
    pub fn component(mut self, def: ComponentDef) -> Self {
        self.components.push(def);
        self
    }

    /// 添加多个组件
    pub fn components(mut self, defs: Vec<ComponentDef>) -> Self {
        self.components.extend(defs);
        self
    }

    /// 设置初始数据
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// 设置样式
    pub fn styles(mut self, styles: SurfaceStyles) -> Self {
        self.styles = Some(styles);
        self
    }

    /// 构建 SurfaceDefinition
    pub fn build(self) -> SurfaceDefinition {
        SurfaceDefinition {
            surface_id: self.surface_id,
            root_id: self.root_id,
            initial_components: self.components,
            initial_data: self.data,
            styles: self.styles,
        }
    }
}

/// 组件构建宏辅助
impl ComponentDef {
    /// 创建 Row 组件
    pub fn row(id: impl Into<String>, children: ChildrenDef) -> Self {
        Self::new(
            id,
            ComponentType::Row(RowProps {
                children,
                distribution: None,
                alignment: None,
                gap: None,
            }),
        )
    }

    /// 创建 Column 组件
    pub fn column(id: impl Into<String>, children: ChildrenDef) -> Self {
        Self::new(
            id,
            ComponentType::Column(ColumnProps {
                children,
                distribution: None,
                alignment: None,
                gap: None,
            }),
        )
    }

    /// 创建 Card 组件
    pub fn card(id: impl Into<String>, child: impl Into<String>) -> Self {
        Self::new(
            id,
            ComponentType::Card(CardProps {
                child: child.into(),
                title: None,
                description: None,
            }),
        )
    }

    /// 创建 Text 组件
    pub fn text(id: impl Into<String>, text: BoundValue) -> Self {
        Self::new(
            id,
            ComponentType::Text(TextProps {
                text,
                variant: None,
            }),
        )
    }

    /// 创建 Text 组件（字面值）
    pub fn text_literal(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self::text(id, BoundValue::string(text))
    }

    /// 创建 Text 组件（路径绑定）
    pub fn text_bound(id: impl Into<String>, path: impl Into<String>) -> Self {
        Self::text(id, BoundValue::path(path))
    }

    /// 创建 Icon 组件
    pub fn icon(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self::new(
            id,
            ComponentType::Icon(IconProps {
                name: BoundValue::string(name),
                size: None,
                color: None,
            }),
        )
    }

    /// 创建 Button 组件
    pub fn button(id: impl Into<String>, child: impl Into<String>, action: Action) -> Self {
        Self::new(
            id,
            ComponentType::Button(ButtonProps {
                child: child.into(),
                action,
                variant: None,
                disabled: None,
            }),
        )
    }

    /// 创建 Badge 组件
    pub fn badge(id: impl Into<String>, text: BoundValue) -> Self {
        Self::new(
            id,
            ComponentType::Badge(BadgeProps {
                text,
                variant: None,
            }),
        )
    }

    /// 创建 List 组件
    pub fn list(id: impl Into<String>, children: ChildrenDef) -> Self {
        Self::new(
            id,
            ComponentType::List(ListProps {
                children,
                direction: None,
                alignment: None,
                gap: None,
            }),
        )
    }

    /// 创建 Alert 组件
    pub fn alert(id: impl Into<String>, message: BoundValue, alert_type: AlertType) -> Self {
        Self::new(
            id,
            ComponentType::Alert(AlertProps {
                message,
                alert_type,
                title: None,
            }),
        )
    }

    /// 创建 Spinner 组件
    pub fn spinner(id: impl Into<String>) -> Self {
        Self::new(id, ComponentType::Spinner(SpinnerProps::default()))
    }

    /// 创建 Empty 组件
    pub fn empty(id: impl Into<String>) -> Self {
        Self::new(id, ComponentType::Empty(EmptyProps::default()))
    }

    /// 创建 Divider 组件
    pub fn divider(id: impl Into<String>) -> Self {
        Self::new(id, ComponentType::Divider(DividerProps::default()))
    }
}

/// Row 属性构建器
impl RowProps {
    pub fn with_distribution(mut self, distribution: Distribution) -> Self {
        self.distribution = Some(distribution);
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn with_gap(mut self, gap: u32) -> Self {
        self.gap = Some(gap);
        self
    }
}

/// Column 属性构建器
impl ColumnProps {
    pub fn with_distribution(mut self, distribution: Distribution) -> Self {
        self.distribution = Some(distribution);
        self
    }

    pub fn with_alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    pub fn with_gap(mut self, gap: u32) -> Self {
        self.gap = Some(gap);
        self
    }
}

/// Card 属性构建器
impl CardProps {
    pub fn with_title(mut self, title: BoundValue) -> Self {
        self.title = Some(title);
        self
    }

    pub fn with_description(mut self, description: BoundValue) -> Self {
        self.description = Some(description);
        self
    }
}

/// Text 属性构建器
impl TextProps {
    pub fn with_variant(mut self, variant: TextVariant) -> Self {
        self.variant = Some(variant);
        self
    }
}

/// Button 属性构建器
impl ButtonProps {
    pub fn with_variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = Some(variant);
        self
    }

    pub fn with_disabled(mut self, disabled: BoundValue) -> Self {
        self.disabled = Some(disabled);
        self
    }
}
