# terminal

<!-- 一旦我所属的文件夹有所变化，请更新我 -->

## 架构说明

内置终端组件，采用**后端预创建 PTY**架构（参考 WaveTerm）。

**核心原则：**
- 后端是会话的唯一真相来源
- PTY 在后端预创建，使用默认大小 (24x80)
- 前端只是"连接"到会话，不负责创建
- resize 只是同步大小，不触发创建

**数据流：**
```
[用户点击新建终端]
       ↓
[前端调用 terminal_create_session]
       ↓
[后端创建 PTY（默认 24x80）]
       ↓
[返回 session_id 给前端]
       ↓
[前端创建 TermWrap，连接到 session_id]
       ↓
[TermWrap 初始化 xterm，监听事件]
       ↓
[首次 fit 后，同步实际大小到后端]
```

## 核心功能

- **PTY 会话管理**: 后端预创建，前端连接
- **实时输入输出**: 通过 Tauri Events 实现
- **自适应大小**: 自动调整终端尺寸，同步到后端
- **xterm.js 渲染**: 高性能终端渲染
- **多标签页**: 支持多个终端会话

## 文件索引

- `index.ts` - 模块导出
- `TerminalPage.tsx` - 终端页面组件（多标签页管理）
- `termwrap.ts` - 终端封装类（连接模式）
- `fitaddon.ts` - 自定义 FitAddon
- `terminal.css` - 终端样式（Tokyo Night 主题）

## 依赖

- `@xterm/xterm` - 终端渲染
- `@xterm/addon-fit` - 自适应大小
- `@xterm/addon-web-links` - 链接支持
- `@/lib/terminal-api` - Tauri 终端 API

## 使用方式

作为内置插件在 `PluginUIRenderer` 中注册：

```typescript
const builtinPluginComponents = {
  "terminal-plugin": TerminalPage,
};
```

## 更新提醒

任何文件变更后，请更新此文档和相关的上级文档。
