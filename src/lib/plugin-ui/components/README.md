# components

<!-- 一旦我所属的文件夹有所变化，请更新我 -->

## 架构说明

插件 UI 标准组件实现。
所有组件基于 shadcn/ui 和 TailwindCSS 构建，提供一致的视觉风格。

## 文件索引

- `layout.tsx` - 布局组件
  - `Row` - 水平布局
  - `Column` - 垂直布局
  - `Card` - 卡片容器
  - `Tabs` - 标签页
- `display.tsx` - 展示组件
  - `Text` - 文本
  - `Icon` - 图标
  - `Badge` - 徽章
  - `Progress` - 进度条
  - `Spinner` - 加载指示器
  - `Empty` - 空状态
  - `Divider` - 分隔线
- `input.tsx` - 输入组件
  - `Button` - 按钮
  - `TextField` - 文本输入
  - `Switch` - 开关
  - `Select` - 下拉选择
- `data.tsx` - 数据展示组件
  - `List` - 列表（支持模板渲染）
  - `KeyValue` - 键值对
  - `Alert` - 提示框
- `index.ts` - 导出入口

## 扩展组件

可通过 `componentRegistry.register()` 注册自定义组件：

```tsx
import { componentRegistry } from '@/lib/plugin-ui';

componentRegistry.register('CustomCard', CustomCardRenderer);
```

## 更新提醒

任何文件变更后，请更新此文档和相关的上级文档。
