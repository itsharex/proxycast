## ProxyCast v0.81.0

### ✨ 新功能

- 主题工作台（Theme Workbench）完整实现：双区域架构（对话区 + 画布区）、上下文管理、版本快照、分支话题
- 主题工作台侧边栏：上下文列表、编排工作台、网络检索、技能面板
- 主题工作台技能面板（ThemeWorkbenchSkillsPanel）：支持技能快捷调用
- 文档画布增强：内容审阅面板、自动续写、文本风格化、版本管理
- 文档导出功能
- 图片上传到会话和文稿导入功能
- 默认技能系统：内置 social_post_with_cover、video_generate、cover_generate、image_generate、library、research、typesetting 等技能
- 主题上下文工作区 hook：管理上下文筛选、激活、检索
- 话题分支看板 hook：支持话题分支创建和切换
- 内容同步增强：支持主题工作台文档状态同步
- 上下文搜索工具：支持素材库和网络检索
- 任务文件画布同步
- 媒体生成统一接口和全局默认设置
- 视频生成设置页面
- 语音设置页面增强
- 工作台右侧面板大幅扩展：支持主题工作台集成
- Provider Pool 服务增强：优化模型路由和池管理
- Execution Run 命令扩展：支持主题工作台状态查询
- 文档编辑器焦点事件管理

### 🐛 修复

- 修复主题工作台内容提取逻辑：AI 普通对话不再被误判为文档内容写入画布
- 修复 default_skills 测试断言逻辑
- 修复 content_cmd 测试中 i32 溢出问题
- 修复 WSL 连接和 live_sync 中的小问题

### 🔧 优化与重构

- 上下文列表 UI 优化：改进间距、字体、自定义 checkbox 样式、选中状态视觉反馈
- InputbarCore 重构：增强输入栏功能和样式
- LayoutTransition 优化
- DocumentToolbar / BubbleToolbar / NotionEditor 增强
- CanvasFactory 重构：支持更多画布类型
- DropdownMenu / Popover / Select UI 组件优化
- I18n 补丁提供者和 DOM 替换器优化
- CrashRecoveryPanel 改进
- WorkbenchStore 扩展
- 系统提示词增强

### 📦 其他

- 新增大量测试覆盖：ThemeWorkbenchSidebar、useThemeContextWorkspace、useTopicBranchBoard、useContentSync、skillCommand、contextSearch、taskFileCanvasSync、extractDocumentContent 等
- E2E 冒烟测试脚本

---

**完整变更**: v0.80.0...v0.81.0
