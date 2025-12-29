# lib

<!-- 一旦我所属的文件夹有所变化，请更新我 -->

## 架构说明

前端工具库和 API 封装层。
包含 Tauri 命令封装、工具函数和服务类。

## 文件索引

- `api/` - API 调用封装
  - `pluginUI.ts` - 插件 UI API（Requirements 3.1）
- `errors/` - 错误处理模块
  - `playwrightErrors.ts` - Playwright 登录错误处理（Requirements 5.1, 5.2, 5.3, 5.4）
- `plugin-ui/` - 插件 UI 系统（基于 A2UI 设计理念）
  - `types.ts` - 类型定义
  - `ComponentRegistry.ts` - 组件注册表
  - `DataStore.ts` - 数据存储
  - `SurfaceManager.ts` - Surface 管理器
  - `PluginUIRenderer.tsx` - 核心渲染器
  - `PluginUIContainer.tsx` - 容器组件
  - `usePluginUI.ts` - React Hook
  - `components/` - 标准组件实现
- `tauri/` - Tauri 命令封装
- `utils/` - 通用工具函数
- `flowEventManager.ts` - 流量事件管理器
- `notificationService.ts` - 通知服务
- `utils.ts` - 通用工具函数

## 更新提醒

任何文件变更后，请更新此文档和相关的上级文档。
