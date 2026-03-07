# 主题工作台功能完善计划

## 目标
完善主题工作台（Theme Workbench）的内容创建功能，增加**添加图片**和**导入文稿**功能。

## 用户需求
1. ✅ 添加图片功能（前后端完整实现）
2. ✅ 导入文稿功能（前后端完整实现）
3. ✅ 保留现有的版本快照和分支话题功能

## 当前状态分析

### 已实现功能
1. ✅ 基础版本快照创建（handleCreateVersionSnapshot）
2. ✅ 上下文管理系统（useThemeContextWorkspace）
3. ✅ 内容模板系统（tech-sharing, trending-topic等）
4. ✅ 侧边栏UI（ThemeWorkbenchSidebar）
5. ✅ 文件上传基础设施（useMaterials hook）

### 待实现功能
1. ❌ "+"按钮的下拉菜单（添加图片、导入文稿选项）
2. ❌ 图片上传和插入功能（前后端）
3. ❌ 文稿导入和解析功能（前后端）

## 实现阶段

### Stage 1: 增强"+"按钮UI
**目标**: 将单一按钮改为下拉菜单，支持多种创建选项

**Success Criteria**:
- "+"按钮点击后显示下拉菜单
- 菜单包含：创建版本快照、添加图片、导入文稿
- 菜单项点击后触发相应功能

**实现步骤**:
1. 修改 ThemeWorkbenchSidebar.tsx，添加 DropdownMenu
2. 添加菜单项：创建版本快照、添加图片、导入文稿
3. 定义回调函数接口

**文件修改**:
- `src/components/agent/chat/components/ThemeWorkbenchSidebar.tsx`

**Status**: Not Started

### Stage 2: 实现添加图片功能（前端）
**目标**: 用户可以选择图片并插入到文档中

**Success Criteria**:
- 点击"添加图片"打开文件选择器
- 支持常见图片格式（jpg, png, gif, webp）
- 图片上传后显示在文档中
- 提供上传进度反馈

**实现步骤**:
1. 在 ThemeWorkbenchSidebar 中添加 onAddImage 回调
2. 在主组件中实现 handleAddImage 函数
3. 调用文件选择器API
4. 调用后端上传接口
5. 将图片URL插入到文档画布

**文件修改**:
- `src/components/agent/chat/components/ThemeWorkbenchSidebar.tsx`
- `src/components/agent/chat/index.tsx`
- `src/lib/api/session-files.ts` (可能需要新增)

**Status**: Not Started

### Stage 3: 实现添加图片功能（后端）
**目标**: 后端接收图片上传请求并存储

**Success Criteria**:
- 接收图片文件上传
- 验证文件类型和大小
- 存储到 session files 或 materials
- 返回图片访问URL

**实现步骤**:
1. 创建 Tauri command: `upload_image_to_session`
2. 实现图片文件验证逻辑
3. 存储图片到本地或云端
4. 返回图片URL

**文件修改**:
- `src-tauri/src/commands/session_files_cmd.rs` (或新建 image_cmd.rs)
- `src-tauri/crates/core/src/session_files/storage.rs`

**Status**: Not Started

### Stage 4: 实现导入文稿功能（前端）
**目标**: 用户可以导入外部文稿到文档画布

**Success Criteria**:
- 点击"导入文稿"打开文件选择器
- 支持 .md, .txt, .docx 格式
- 文稿内容解析后加载到编辑器
- 提供导入进度反馈

**实现步骤**:
1. 在 ThemeWorkbenchSidebar 中添加 onImportDocument 回调
2. 在主组件中实现 handleImportDocument 函数
3. 调用文件选择器API
4. 调用后端解析接口
5. 将解析后的内容加载到文档画布

**文件修改**:
- `src/components/agent/chat/components/ThemeWorkbenchSidebar.tsx`
- `src/components/agent/chat/index.tsx`
- `src/lib/api/document-import.ts` (新建)

**Status**: Not Started

### Stage 5: 实现导入文稿功能（后端）
**目标**: 后端解析不同格式的文稿文件

**Success Criteria**:
- 接收文件路径或文件内容
- 解析 Markdown (.md)
- 解析纯文本 (.txt)
- 解析 Word 文档 (.docx)
- 返回统一的 Markdown 格式

**实现步骤**:
1. 创建 Tauri command: `import_document`
2. 实现 Markdown 解析器
3. 实现纯文本解析器
4. 实现 Word 文档解析器（使用 docx-rs 或类似库）
5. 统一输出格式

**文件修改**:
- `src-tauri/src/commands/document_import_cmd.rs` (新建)
- `src-tauri/crates/services/src/document_import_service.rs` (新建)
- `src-tauri/Cargo.toml` (添加依赖)

**Status**: Not Started

## 技术细节

### 前端技术栈
- React + TypeScript
- styled-components
- Tauri API (文件选择器)
- lucide-react (图标)

### 后端技术栈
- Rust + Tauri
- docx-rs (Word 文档解析)
- markdown (Markdown 解析)
- tokio (异步IO)

### 文件格式支持
**图片格式**:
- JPEG (.jpg, .jpeg)
- PNG (.png)
- GIF (.gif)
- WebP (.webp)

**文稿格式**:
- Markdown (.md)
- 纯文本 (.txt)
- Word 文档 (.docx)

### 依赖关系
- Stage 1 独立实现（UI基础）
- Stage 2 依赖 Stage 1 和 Stage 3
- Stage 3 独立实现（后端图片）
- Stage 4 依赖 Stage 1 和 Stage 5
- Stage 5 独立实现（后端文稿）

## 注意事项
1. 文件大小限制：图片 < 10MB，文稿 < 5MB
2. 安全性：验证文件类型，防止恶意文件上传
3. 错误处理：提供清晰的错误提示
4. 用户体验：显示上传/导入进度
5. 性能优化：大文件异步处理
6. 兼容性：确保与现有功能不冲突
