# Playwright MCP 续测与 E2E 指南

## 目的

本文件用于指导 AI Agent 在 ProxyCast 中继续进行浏览器端 E2E 测试，特别适用于以下场景：

- 用户说“继续测试”“继续复现”“继续用 Playwright MCP 验证”
- 需要复用当前浏览器标签页和已有页面状态
- 需要排查浏览器模式下的 DevBridge、mock fallback、控制台报错

## 适用边界

- **优先使用 Playwright MCP** 做交互验证，不优先编写新的本地 Playwright 测试文件
- **浏览器模式默认首页不变**，从 `http://127.0.0.1:1420/` 进入
- **能走真实后端就走真实后端**；浏览器模式暂不支持或尚未桥接的能力，允许继续走 mock
- 如果只是模块级代码修改、并不需要真实页面交互，优先跑最小单测，不要强行启动整条 E2E

## AGENTS.md 最佳实践（本仓库落地版）

为避免 `AGENTS.md` 膨胀，遵循以下组织方式：

1. **根 `AGENTS.md` 只保留仓库级规则和入口索引**
2. **长流程文档放 `docs/aiprompts/`**，例如本文件
3. **只有某个子目录存在长期稳定、强作用域规则时，才新增子目录 `AGENTS.md`**
4. **临时排障记录不要写进 `AGENTS.md`**，应写入普通文档或直接在任务对话里说明

## 启动方式

### 推荐启动命令

```bash
npm run tauri:dev:headless
```

用途：
- 启动前端 dev server
- 启动 Tauri headless 调试环境
- 启动浏览器模式所需的 DevBridge
- 便于 Playwright MCP 访问 `http://127.0.0.1:1420/`

### 针对性前端校验

```bash
npm test -- src/lib/dev-bridge/safeInvoke.test.ts src/lib/tauri-mock/core.test.ts
```

适用时机：
- 修改了 `safeInvoke`
- 修改了 `src/lib/tauri-mock/`
- 修改了浏览器模式 bridge/mock 优先级

## 继续测试的标准流程

### 1. 先确认当前 Playwright 会话是否可复用

优先顺序：

1. 调用 `browser_tabs` 查看当前标签页
2. 如果已有 `ProxyCast` 标签页，先查看当前 URL、标题、页面状态
3. 如果页面已漂移到旧状态，直接重新导航到 `http://127.0.0.1:1420/`

建议：
- **继续测试优先复用当前标签页**，避免无意义重复建页
- **如果控制台历史噪音太多，刷新页面重新计数**

### 2. 进入页面前先检查加载状态

推荐动作：

1. 打开页面后等待“正在加载...”消失
2. 使用 `browser_snapshot` 确认首页核心元素已出现
3. 立刻检查一次 `browser_console_messages(level=error)`

通过标准：
- 首页成功加载
- 默认首页可交互
- 初始控制台 error 为 0；若非 0，需要先定位是否为 bridge 缺口

### 3. 交互时优先使用稳定定位方式

遵循 Playwright 官方最佳实践：

- 优先使用 **角色、名称、可见文本** 定位
- 优先使用 Playwright 自带等待与 web-first 断言
- **不要依赖固定 sleep** 代替状态判断
- 点击前先确认元素可见、可交互

本仓库中建议优先使用：
- `button` + 中文名称
- 页面中明确可见的标题文本
- `browser_snapshot` 返回的 ref 作为精确交互目标

## ProxyCast 推荐 E2E 主路径

### 首页基础验证

1. 打开 `http://127.0.0.1:1420/`
2. 等待默认首页加载完成
3. 验证首页主导航可见：如“首页”“社媒内容”“设置”
4. 检查 `browser_console_messages(level=error)` 为 0

### 社媒内容工作流验证

1. 点击 `社媒内容`
2. 如果没有项目：点击 `新建项目`
3. 如果已有项目：直接选择目标项目
4. 点击 `新建文稿`
5. 选择 `新开帖子（创建新文稿）`
6. 点击 `确认生成`
7. 验证页面出现 `Theme Workbench` 或相关工作台内容
8. 再次检查 `browser_console_messages(level=error)`

### 素材页验证

1. 从社媒内容项目进入 `素材`
2. 验证素材列表可加载
3. 验证素材计数、列表项或空状态正常显示
4. 检查控制台无新增 error

## 每一步都要记录什么

执行 Playwright MCP 续测时，至少记录以下事实：

- 当前页面 URL
- 当前关键可见文本
- 是否走到了真实 bridge
- 是否触发了 mock fallback
- 控制台 error 数量
- 如失败，明确失败命令名（例如某个 invoke command）

推荐结论格式：

- 页面是否可打开
- 业务流是否走通
- 控制台是否归零
- 新暴露的命令缺口是什么
- 该缺口适合补真实 bridge 还是补 mock

## 浏览器模式常见故障与处理

### 1. `Cannot read properties of undefined (reading 'invoke')`

通常表示：
- 浏览器里加载了真实 Tauri API 包
- 没有走 web mock / HTTP bridge 链路

优先排查：
- 是否使用了浏览器模式专用启动方式
- Vite 是否正确走了 web alias
- 当前页面是否需要强制刷新以拿到最新前端代码

### 2. `[DevBridge] 未知命令`

说明：
- 前端已调用某命令
- 浏览器 bridge 分发器没有实现

处理顺序：
1. 先判断该命令是否应走真实后端
2. 如果该能力在浏览器模式下不是关键阻塞项，可加入 mock 优先集合
3. 如果该命令属于核心业务路径，优先补 `dispatcher.rs`

### 3. `Failed to fetch`

常见原因：
- DevBridge 没启动
- 3030 端口不可用
- 前端先于 bridge 就绪开始调用

处理建议：
- 确认 `tauri:dev:headless` 已启动
- 检查 bridge 健康接口
- 刷新页面后复测，排除启动时序问题

### 4. UI 已可用但控制台仍报错

说明：
- 页面可能依赖 fallback mock 继续运行
- 但仍有命令先打到了 bridge 并报 unknown command

处理建议：
- 若该命令属于浏览器模式可接受的降级能力，加入 mock 优先列表
- 若该命令属于当前主路径必须能力，补真实 bridge

## 何时补 mock，何时补真实 bridge

### 优先补真实 bridge

适用于：
- 当前主路径必须命令
- 明确已有后端实现
- 返回结构简单稳定
- 不涉及复杂流式事件或强原生依赖

### 优先补 mock

适用于：
- 浏览器模式不支持的原生能力
- 非主路径功能
- 高频噪音命令，但不影响主流程完成
- 流式/系统级能力，短期内 bridge 成本高于收益

## 结果判定标准

一次“继续测试”完成后，至少满足以下之一：

1. **主路径走通且控制台 error 归零**
2. **主路径走通，且剩余错误已被明确归类为非阻塞项**
3. **已定位新的 bridge 缺口，并给出下一步最小修复点**

## 给后续 Agent 的交接要求

如果本轮没有完全收口，请在结论中明确留下：

- 当前停留页面
- 已完成的业务步骤
- 最新暴露的命令缺口
- 推荐下一步先补 mock 还是先补 bridge
- 下一轮建议的 Playwright 复测路径
