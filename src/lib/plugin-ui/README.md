# plugin-ui

<!-- 一旦我所属的文件夹有所变化，请更新我 -->

## 架构说明

基于 A2UI 设计理念的声明式插件 UI 系统。

核心思想：
- **安全如数据，表达如代码**：插件只能声明 UI 结构，不能执行任意代码
- **声明式 JSON 格式**：插件通过 JSON 描述 UI 意图，宿主应用负责渲染
- **组件目录（Catalog）机制**：预定义可用组件集，插件只能使用目录中的组件
- **数据绑定分离**：UI 结构与数据模型分离，支持增量更新

## 文件索引

- `types.ts` - 类型定义（组件、消息、状态等）
- `ComponentRegistry.ts` - 组件注册表，管理可用组件白名单
- `DataStore.ts` - 数据存储，支持路径访问和更新
- `SurfaceManager.ts` - Surface 管理器，处理 UI 消息
- `PluginUIRenderer.tsx` - 核心渲染器组件
- `PluginUIContainer.tsx` - 封装容器，含加载/错误状态
- `usePluginUI.ts` - React Hook，管理插件 UI 状态
- `index.ts` - 导出入口
- `components/` - 标准组件实现
  - `layout.tsx` - 布局组件（Row, Column, Card, Tabs）
  - `display.tsx` - 展示组件（Text, Icon, Badge, Progress 等）
  - `input.tsx` - 输入组件（Button, TextField, Switch, Select）
  - `data.tsx` - 数据组件（List, KeyValue, Alert）
  - `index.ts` - 组件导出

## 使用示例

```tsx
import { PluginUIContainer } from '@/lib/plugin-ui';

function PluginDetailPage({ pluginId }) {
  return (
    <div>
      <h2>插件详情</h2>
      <PluginUIContainer pluginId={pluginId} />
    </div>
  );
}
```

## 设计文档

详细设计见 `docs/plugin-ui-design.md`

## 更新提醒

任何文件变更后，请更新此文档和相关的上级文档。
