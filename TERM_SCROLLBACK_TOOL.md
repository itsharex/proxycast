# Terminal Scrollback Tool 实现文档

## 概述

`term_get_scrollback` 工具允许 AI 读取终端的输出历史，而不是直接执行命令。这是一个**只读工具**，完全避免了 AI 重复执行命令的问题。

## 架构设计

### 后端（Rust）

1. **TermScrollbackTool** (`src-tauri/src/agent/tools/term_scrollback.rs`)
   - 实现 `Tool` trait
   - 使用响应通道机制（类似 TerminalTool）
   - 通过 Tauri 事件系统与前端通信

2. **全局实例管理**
   - 使用 `once_cell::sync::Lazy` 创建全局单例
   - 在 `setup.rs` 和 `runner.rs` 中初始化 AppHandle
   - 提供 `get_term_scrollback_tool()` 获取全局实例

3. **Tauri 命令**
   - `agent_term_scrollback_response`: 接收前端的响应

### 前端（TypeScript/React）

1. **API 层** (`src/lib/api/agent.ts`)
   - `TermScrollbackRequest`: 请求类型
   - `TermScrollbackResponse`: 响应类型
   - `sendTermScrollbackResponse()`: 发送响应函数

2. **Hook 层** (`src/components/terminal/ai/useTerminalAI.ts`)
   - 监听 `term_get_scrollback_request` 事件
   - 读取终端输出历史
   - 发送响应给后端

## 工作流程

```
AI Agent
  ↓ (调用 term_get_scrollback 工具)
TermScrollbackTool
  ↓ (发送 term_get_scrollback_request 事件)
前端 useTerminalAI
  ↓ (读取终端输出)
  ↓ (调用 sendTermScrollbackResponse)
Tauri 命令 agent_term_scrollback_response
  ↓ (调用 handle_term_scrollback_response)
TermScrollbackTool
  ↓ (通过响应通道返回结果)
AI Agent (收到终端输出)
```

## 工具参数

```json
{
  "session_id": "终端会话 ID",
  "line_start": 0,  // 可选，起始行号（从 0 开始）
  "count": 50       // 可选，读取行数
}
```

## 工具描述（给 AI 的说明）

```
Read terminal output history without executing commands.

This tool allows you to view the terminal's scrollback buffer (output history)
without executing any commands. Use this to:
- Check the results of previously executed commands
- Review terminal output before suggesting next steps
- Understand the current state of the terminal session

Parameters:
- session_id: Terminal session ID (required)
- line_start: Starting line number (optional, default: 0)
- count: Number of lines to read (optional, default: all lines)

Returns the terminal output as plain text.

IMPORTANT: This is a READ-ONLY tool. It does NOT execute commands.
```

## 使用场景

### 场景 1：纯只读模式（推荐）

**配置：**
- 移除 `terminal` 工具
- 只保留 `term_get_scrollback` 工具

**优势：**
- AI 只能读取输出，不能执行命令
- 完全避免重复执行问题
- 用户完全控制命令执行

**工作流程：**
1. 用户手动在终端执行命令
2. AI 使用 `term_get_scrollback` 读取输出
3. AI 根据输出提供建议
4. 用户决定是否执行 AI 的建议

### 场景 2：混合模式（当前实现）

**配置：**
- 保留 `terminal` 工具（需要审批）
- 添加 `term_get_scrollback` 工具

**优势：**
- AI 可以建议命令（需要审批）
- AI 也可以读取历史输出
- 灵活性更高

**工作流程：**
1. AI 建议命令（通过 `terminal` 工具）
2. 用户审批并执行
3. AI 使用 `term_get_scrollback` 读取输出
4. AI 根据输出继续工作

## 测试步骤

### 1. 编译项目

```bash
cargo build --manifest-path src-tauri/Cargo.toml
```

### 2. 启动应用

```bash
npm run tauri dev
```

### 3. 测试工具

1. 打开终端
2. 执行一些命令（例如：`ls`, `pwd`, `echo hello`）
3. 打开 AI 面板
4. 发送消息：`请读取终端的输出历史`
5. AI 应该使用 `term_get_scrollback` 工具读取输出
6. 检查 AI 是否正确显示了终端输出

### 4. 验证日志

**后端日志：**
```
[TermScrollbackTool] 创建全局实例
[TermScrollbackTool] 设置全局 AppHandle
[TermScrollbackTool] AppHandle 设置成功，已验证
[TermScrollbackTool] 请求获取滚动缓冲区: session_id=xxx, request_id=xxx
[TermScrollbackTool] 已发送请求到前端: xxx
[TermScrollbackTool] 收到响应: request_id=xxx, success=true
```

**前端日志：**
```
[useTerminalAI] 收到终端滚动缓冲区请求: {...}
[useTerminalAI] 已发送滚动缓冲区响应: 0-50/100 行
```

## 故障排查

### 问题 1：AppHandle 未设置

**症状：**
```
[TermScrollbackTool] 警告：AppHandle 未设置！工具将无法正常工作
```

**解决方案：**
- 检查 `setup.rs` 和 `runner.rs` 中是否调用了 `set_term_scrollback_tool_app_handle()`
- 确保在工具注册之前设置 AppHandle

### 问题 2：前端未收到事件

**症状：**
- 后端发送了事件，但前端没有日志

**解决方案：**
- 检查 `useTerminalAI` 是否正确监听 `term_get_scrollback_request` 事件
- 确保 `terminalSessionId` 不为空
- 检查浏览器控制台是否有错误

### 问题 3：响应超时

**症状：**
```
[TermScrollbackTool] 请求超时: xxx
```

**解决方案：**
- 检查前端是否正确发送响应
- 增加超时时间：`TermScrollbackTool::new().with_timeout(60)`
- 检查 `getTerminalOutput()` 函数是否正常工作

## 下一步改进

1. **添加过滤功能**
   - 支持正则表达式过滤
   - 支持关键词搜索

2. **添加格式化选项**
   - 支持 ANSI 颜色代码
   - 支持纯文本输出

3. **添加缓存机制**
   - 缓存最近的输出
   - 减少重复读取

4. **添加增量读取**
   - 只读取新增的输出
   - 支持实时监控

## 参考资料

- [Waveterm Terminal AI](https://github.com/wavetermdev/waveterm)
- [Tauri Event System](https://tauri.app/v1/guides/features/events/)
- [Tokio Oneshot Channel](https://docs.rs/tokio/latest/tokio/sync/oneshot/)
