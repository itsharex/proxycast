## ProxyCast v0.83.2

### ✨ 新功能
- 新增跨平台应用路径解析模块 `app_paths`，支持 macOS/Windows 目录迁移
- Agent 事件转换器增强，支持更多事件类型处理
- Agent 请求工具策略扩展，新增策略规则
- 流式渲染器增强，新增流诊断工具和 Provider 模型兼容性检测
- 终端 AI 模式选择器功能增强
- OpenClaw 页面功能扩展
- Windows 启动命令模块增强

### 🐛 修复
- 修复 useMemo 依赖缺失导致的 React Hook 警告
- 修复 Kiro Provider 凭证处理逻辑
- 修复心跳服务适配器和心跳命令的稳定性问题
- 修复日志模块和遥测日志的路径处理
- 修复数据库模块初始化问题
- 修复托盘菜单事件处理逻辑

### 🔧 优化与重构
- Provider 模型选择器组件重构，提升可维护性
- ModelSelector 组件优化，增加测试覆盖
- 通用聊天 useProvider Hook 重构
- Workbench 页面布局优化
- 频道设置页面改进
- 终端工作区组件优化
- 语音润色模型选择器改进
- useProjects Hook 优化

### 📦 其他
- 新增多个组件单元测试（StreamingRenderer、ProviderModelSelector、TerminalAIModeSelector、ModelSelector）
- 新增流诊断和 Provider 模型兼容性工具测试
- Cargo.lock 依赖更新

---

**完整变更**: v0.83.1...v0.83.2
