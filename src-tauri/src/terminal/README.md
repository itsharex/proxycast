# terminal

<!-- 一旦我所属的文件夹有所变化，请更新我 -->

## 架构说明

终端核心模块，采用**后端预创建 PTY**架构（参考 WaveTerm）。

**核心原则：**
- 后端是会话的唯一真相来源
- PTY 使用默认大小 (24x80) 预创建
- 前端连接后通过 resize 同步实际大小
- 通过 Tauri Commands 和 Events 暴露给前端

## 核心功能

- **PTY 管理**: 创建和管理伪终端进程（默认大小预创建）
- **会话管理**: 多会话支持，生命周期管理
- **实时输出**: 通过 Tauri Events 推送终端输出
- **状态通知**: 会话状态变化事件

## 文件索引

- `mod.rs` - 模块入口和类型导出
- `error.rs` - 错误类型定义
- `events.rs` - Tauri 事件定义（terminal:output, terminal:status）
- `pty_session.rs` - PTY 会话封装（支持默认大小创建）
- `session_manager.rs` - 会话管理器
- `tests.rs` - 单元测试

## 命令接口

| 命令 | 描述 | 参数 |
|------|------|------|
| `terminal_create_session` | 创建终端会话（默认大小） | 无 |
| `terminal_write` | 向终端发送输入 | `session_id`, `data` |
| `terminal_resize` | 调整终端大小 | `session_id`, `rows`, `cols` |
| `terminal_close` | 关闭终端会话 | `session_id` |
| `terminal_list_sessions` | 获取所有会话列表 | 无 |
| `terminal_get_session` | 获取单个会话信息 | `session_id` |

## 事件定义

| 事件名 | 描述 | 数据结构 |
|--------|------|----------|
| `terminal:output` | 终端输出数据 | `{ session_id, data }` |
| `terminal:status` | 会话状态变化 | `{ session_id, status, exit_code?, error? }` |

## 常量

- `DEFAULT_ROWS`: 默认终端行数 (24)
- `DEFAULT_COLS`: 默认终端列数 (80)

## 依赖

- `portable-pty` - 跨平台 PTY 支持

## 更新提醒

任何文件变更后，请更新此文档和相关的上级文档。
