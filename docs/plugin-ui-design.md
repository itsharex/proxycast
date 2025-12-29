# ProxyCast Plugin UI 系统设计

## 概述

借鉴 A2UI 的设计理念，为 ProxyCast 设计一套声明式的插件 UI 系统。核心思想是：

- **安全如数据，表达如代码**：插件只能声明 UI 结构，不能执行任意代码
- **声明式 JSON 格式**：插件通过 JSON 描述 UI 意图，宿主应用负责渲染
- **组件目录（Catalog）机制**：预定义可用组件集，插件只能使用目录中的组件
- **数据绑定分离**：UI 结构与数据模型分离，支持增量更新

## 架构设计

```
┌─────────────────────────────────────────────────────────────────┐
│                        ProxyCast Host                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Plugin UI Renderer                    │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │   │
│  │  │  Component  │  │    Data     │  │     Event       │  │   │
│  │  │  Registry   │  │   Store     │  │    Handler      │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              ▲                                  │
│                              │ JSON Messages                    │
│  ┌───────────────────────────┼─────────────────────────────┐   │
│  │                    Plugin Bridge                         │   │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  │   │
│  │  │   Tauri     │  │   Schema    │  │    Message      │  │   │
│  │  │   IPC       │  │  Validator  │  │    Router       │  │   │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                              ▲
                              │
              ┌───────────────┴───────────────┐
              │         Plugin (Rust)          │
              │  ┌─────────────────────────┐  │
              │  │    UI Declaration API    │  │
              │  │  - surface_update()      │  │
              │  │  - data_update()         │  │
              │  │  - begin_rendering()     │  │
              │  └─────────────────────────┘  │
              └───────────────────────────────┘
```

## 核心概念

### 1. Surface（渲染表面）

每个插件可以拥有一个或多个 Surface，代表独立的 UI 区域：

```typescript
interface Surface {
  surfaceId: string;           // 唯一标识
  pluginId: string;            // 所属插件
  rootComponentId: string;     // 根组件 ID
  components: Map<string, Component>;  // 组件缓冲区
  dataModel: Record<string, any>;      // 数据模型
  styles?: SurfaceStyles;      // 样式配置
}
```

### 2. Component Catalog（组件目录）

预定义的安全组件集，插件只能使用这些组件：

```typescript
// 标准组件目录
const StandardCatalog = {
  // 布局组件
  Row: { children: 'ComponentRef[]', gap?: 'number', align?: 'Alignment' },
  Column: { children: 'ComponentRef[]', gap?: 'number', align?: 'Alignment' },
  Card: { child: 'ComponentRef', title?: 'BoundValue<string>' },
  Tabs: { items: 'TabItem[]' },
  
  // 展示组件
  Text: { text: 'BoundValue<string>', variant?: 'TextVariant' },
  Icon: { name: 'IconName', size?: 'number', color?: 'string' },
  Badge: { text: 'BoundValue<string>', variant?: 'BadgeVariant' },
  Progress: { value: 'BoundValue<number>', max?: 'number' },
  
  // 输入组件
  Button: { child: 'ComponentRef', action: 'Action', variant?: 'ButtonVariant' },
  TextField: { label: 'BoundValue<string>', value: 'BoundValue<string>' },
  Switch: { label: 'BoundValue<string>', checked: 'BoundValue<boolean>' },
  Select: { options: 'SelectOption[]', value: 'BoundValue<string>' },
  
  // 数据展示
  Table: { columns: 'TableColumn[]', data: 'BoundValue<any[]>' },
  List: { children: 'ChildrenDef', direction?: 'Direction' },
  KeyValue: { items: 'KeyValueItem[]' },
  
  // 反馈组件
  Alert: { message: 'BoundValue<string>', type: 'AlertType' },
  Spinner: { size?: 'number' },
  Empty: { description?: 'BoundValue<string>' },
};
```

### 3. 消息协议

#### Server → Client 消息

```typescript
// 组件更新
interface SurfaceUpdate {
  surfaceId: string;
  components: ComponentDef[];
}

// 数据更新
interface DataModelUpdate {
  surfaceId: string;
  path?: string;  // JSONPath，如 '/credentials/0/status'
  contents: DataEntry[];
}

// 开始渲染
interface BeginRendering {
  surfaceId: string;
  root: string;  // 根组件 ID
  catalogId?: string;
  styles?: SurfaceStyles;
}

// 删除 Surface
interface DeleteSurface {
  surfaceId: string;
}
```

#### Client → Server 消息

```typescript
// 用户操作
interface UserAction {
  name: string;           // 操作名称
  surfaceId: string;
  sourceComponentId: string;
  context: Record<string, any>;  // 解析后的上下文数据
  timestamp: string;
}
```

### 4. 数据绑定

支持字面值和路径绑定：

```typescript
type BoundValue<T> = 
  | { literal: T }                    // 字面值
  | { path: string }                  // 数据路径
  | { literal: T; path: string };     // 初始化 + 绑定

// 示例
const textComponent = {
  id: 'status-text',
  component: {
    Text: {
      text: { path: '/credential/status' },  // 绑定到数据模型
      variant: 'body'
    }
  }
};
```

## 实现方案

### 前端：React Renderer

```
src/lib/plugin-ui/
├── index.ts                 # 导出入口
├── types.ts                 # 类型定义
├── PluginUIRenderer.tsx     # 主渲染器组件
├── PluginSurface.tsx        # Surface 容器
├── ComponentRegistry.ts     # 组件注册表
├── DataStore.ts             # 数据存储
├── MessageHandler.ts        # 消息处理
└── components/              # 标准组件实现
    ├── layout/
    │   ├── Row.tsx
    │   ├── Column.tsx
    │   ├── Card.tsx
    │   └── Tabs.tsx
    ├── display/
    │   ├── Text.tsx
    │   ├── Icon.tsx
    │   ├── Badge.tsx
    │   └── Progress.tsx
    ├── input/
    │   ├── Button.tsx
    │   ├── TextField.tsx
    │   ├── Switch.tsx
    │   └── Select.tsx
    └── data/
        ├── Table.tsx
        ├── List.tsx
        └── KeyValue.tsx
```

### 后端：Rust Plugin API

```rust
// src-tauri/src/plugins/ui_api.rs

/// 插件 UI 声明 API
pub trait PluginUI {
    /// 获取插件的 Surface 定义
    fn get_surfaces(&self) -> Vec<SurfaceDefinition>;
    
    /// 处理用户操作
    fn handle_action(&mut self, action: UserAction) -> Result<Vec<UIMessage>>;
}

/// UI 消息类型
pub enum UIMessage {
    SurfaceUpdate(SurfaceUpdate),
    DataModelUpdate(DataModelUpdate),
    BeginRendering(BeginRendering),
    DeleteSurface(DeleteSurface),
}

/// Surface 定义
pub struct SurfaceDefinition {
    pub surface_id: String,
    pub initial_components: Vec<ComponentDef>,
    pub initial_data: serde_json::Value,
    pub root_id: String,
}
```

## 使用示例

### 插件端（Rust）

```rust
impl PluginUI for CredentialMonitorPlugin {
    fn get_surfaces(&self) -> Vec<SurfaceDefinition> {
        vec![SurfaceDefinition {
            surface_id: "credential-monitor".into(),
            root_id: "root".into(),
            initial_components: vec![
                component!("root", Column {
                    children: explicit_list!["header", "credential-list"],
                    gap: 16
                }),
                component!("header", Row {
                    children: explicit_list!["title", "refresh-btn"],
                    align: "spaceBetween"
                }),
                component!("title", Text {
                    text: literal!("凭证监控"),
                    variant: "h3"
                }),
                component!("refresh-btn", Button {
                    child: "refresh-icon",
                    action: action!("refresh")
                }),
                component!("refresh-icon", Icon { name: "refresh" }),
                component!("credential-list", List {
                    children: template!("credential-item", "/credentials"),
                    direction: "vertical"
                }),
                // 模板组件
                component!("credential-item", Card {
                    child: "item-content"
                }),
                component!("item-content", Row {
                    children: explicit_list!["item-name", "item-status"]
                }),
                component!("item-name", Text {
                    text: path!("name")  // 相对路径，从列表项数据解析
                }),
                component!("item-status", Badge {
                    text: path!("status"),
                    variant: path!("statusVariant")
                }),
            ],
            initial_data: json!({
                "credentials": []
            }),
        }]
    }
    
    fn handle_action(&mut self, action: UserAction) -> Result<Vec<UIMessage>> {
        match action.name.as_str() {
            "refresh" => {
                let credentials = self.fetch_credentials()?;
                Ok(vec![UIMessage::DataModelUpdate(DataModelUpdate {
                    surface_id: "credential-monitor".into(),
                    path: Some("/credentials".into()),
                    contents: credentials.into_data_entries(),
                })])
            }
            _ => Ok(vec![])
        }
    }
}
```

### 宿主端（React）

```tsx
// 在插件详情页使用
function PluginDetailPage({ pluginId }: { pluginId: string }) {
  return (
    <div className="plugin-detail">
      <PluginInfo pluginId={pluginId} />
      
      {/* 插件 UI 渲染区域 */}
      <PluginUIRenderer 
        pluginId={pluginId}
        onAction={(action) => invoke('plugin_handle_action', { pluginId, action })}
      />
    </div>
  );
}
```

## 安全考虑

1. **组件白名单**：只允许使用预定义的组件类型
2. **Schema 验证**：所有消息必须通过 JSON Schema 验证
3. **沙箱隔离**：每个插件的 Surface 相互隔离
4. **Action 审计**：记录所有用户操作，支持权限控制
5. **资源限制**：限制组件数量、数据大小等

## 扩展机制

### 自定义组件注册

允许宿主应用注册额外的组件：

```typescript
// 注册自定义组件
componentRegistry.register('CredentialCard', CredentialCardComponent, {
  schema: {
    credential: { type: 'object', required: true },
    onRefresh: { type: 'action' }
  }
});
```

### 主题支持

通过 Surface styles 支持主题定制：

```typescript
interface SurfaceStyles {
  primaryColor?: string;
  font?: string;
  borderRadius?: number;
  // ... 更多样式属性
}
```

## 迁移路径

1. **Phase 1**：实现核心渲染器和基础组件
2. **Phase 2**：添加数据绑定和事件处理
3. **Phase 3**：迁移现有插件 UI 到新系统
4. **Phase 4**：支持自定义组件扩展

## 与 A2UI 的差异

| 特性 | A2UI | ProxyCast Plugin UI |
|------|------|---------------------|
| 传输方式 | SSE/JSONL 流 | Tauri IPC |
| 渲染框架 | Lit/Angular/Flutter | React |
| 组件风格 | Material Design | TailwindCSS/shadcn |
| 数据更新 | 增量流式 | 批量更新 |
| 使用场景 | 跨平台 Agent UI | 桌面应用插件 |


## 实时更新：Tauri 事件推送

插件可以通过 Tauri 事件系统向前端推送 UI 更新，实现实时数据刷新。

### 事件发射器

```rust
use crate::plugin::{PluginUIEmitter, UIMessage, DataModelUpdate, DataEntry};

// 在 Tauri 命令或服务中使用
fn update_plugin_ui(emitter: &PluginUIEmitter, plugin_id: &str) {
    // 发送数据更新
    let update = DataModelUpdate {
        surface_id: "my-surface".into(),
        path: Some("/stats".into()),
        contents: vec![
            DataEntry::number("count", 42.0),
            DataEntry::string("status", "healthy"),
        ],
    };
    
    emitter.emit_data_update(plugin_id, update).unwrap();
}
```

### 前端监听

前端通过 `usePluginUI` Hook 自动监听 `plugin-ui-message` 事件：

```typescript
// 自动处理，无需手动监听
const { surfaces, handleAction } = usePluginUI({ pluginId: 'my-plugin' });
```

### 事件载荷格式

```typescript
interface PluginUIEventPayload {
  pluginId: string;
  message: UIMessage;  // SurfaceUpdate | DataModelUpdate | BeginRendering | DeleteSurface
}
```

## 示例插件：凭证监控

完整示例见 `src-tauri/src/plugin/examples/credential_monitor.rs`：

```rust
use crate::plugin::{PluginUI, SurfaceDefinition, ComponentDef, ChildrenDef, BoundValue};

struct CredentialMonitorPlugin { /* ... */ }

impl PluginUI for CredentialMonitorPlugin {
    fn get_surfaces(&self) -> Vec<SurfaceDefinition> {
        vec![SurfaceDefinition {
            surface_id: "credential-monitor".into(),
            root_id: "root".into(),
            initial_components: vec![
                ComponentDef::column("root", ChildrenDef::explicit(vec!["header", "list"])),
                ComponentDef::text_literal("header", "凭证监控"),
                ComponentDef::list("list", ChildrenDef::template("item", "/credentials")),
                // ... 更多组件
            ],
            initial_data: json!({ "credentials": [] }),
            styles: None,
        }]
    }

    async fn handle_action(&mut self, action: UserAction) -> Result<Vec<UIMessage>, PluginError> {
        match action.name.as_str() {
            "refresh" => {
                // 返回数据更新消息
                Ok(vec![UIMessage::DataModelUpdate(/* ... */)])
            }
            _ => Ok(vec![])
        }
    }
}
```

## 下一步计划

1. **更多组件**：Table、Tabs、Modal 等复杂组件
2. **表单验证**：支持 TextField 的验证规则
3. **主题系统**：更完善的样式定制能力
4. **插件市场**：支持从远程加载插件 UI 定义
