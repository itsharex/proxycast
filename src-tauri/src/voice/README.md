# voice/ - 语音输入模块

语音输入功能的 Tauri 后端模块，提供全局快捷键、悬浮窗、ASR 识别、LLM 润色等功能。

## 文件索引

| 文件 | 说明 |
|------|------|
| `mod.rs` | 模块入口，导出子模块 |
| `asr_service.rs` | ASR 服务，统一管理本地 Whisper 和云端 ASR |
| `commands.rs` | Tauri 命令，供前端调用 |
| `config.rs` | 配置管理，读写语音输入配置 |
| `output_service.rs` | 文字输出服务，模拟键盘输入和剪贴板 |
| `processor.rs` | LLM 润色处理，调用本地 API 服务器 |
| `recording_service.rs` | 录音服务，使用独立线程 + channel 通信 |
| `shortcut.rs` | 全局快捷键管理 |
| `window.rs` | 悬浮窗管理 |

## 录音服务架构

由于 `cpal::Stream` 不实现 `Send` trait，无法直接在 Tauri 的 async 命令中使用。
录音服务采用**独立线程 + channel 通信**的方案：

```
┌─────────────────┐     Command      ┌─────────────────┐
│  Tauri Command  │ ───────────────> │  Recording      │
│  (async)        │                  │  Thread         │
│                 │ <─────────────── │  (owns Stream)  │
└─────────────────┘     Response     └─────────────────┘
```

### 录音命令

| 命令 | 说明 |
|------|------|
| `start_recording` | 开始录音 |
| `stop_recording` | 停止录音，返回音频数据 |
| `cancel_recording` | 取消录音 |
| `get_recording_status` | 获取录音状态（是否录音中、音量、时长）|

## 依赖关系

```
voice/
├── asr_service.rs ──→ voice-core (WhisperTranscriber, AsrClient)
├── output_service.rs ──→ voice-core (OutputHandler)
├── recording_service.rs ──→ voice-core (threaded_recorder + Tauri State 包装)
├── processor.rs ──→ 本地 API 服务器 (LLM 润色)
└── commands.rs ──→ 上述所有服务
```

## ASR 服务支持

| Provider | 状态 | 说明 |
|----------|------|------|
| Whisper Local | ✅ | 本地离线识别，需下载模型文件 |
| OpenAI Whisper | ✅ | 云端 API，支持自定义 base_url |
| 百度语音 | ✅ | 云端 API |
| 讯飞语音 | ✅ | WebSocket 流式识别 |

### 云端回退机制

当云端 ASR 服务（OpenAI、百度、讯飞）失败时，系统会自动回退到本地 Whisper 进行识别：

1. 首先尝试用户选择的云端服务
2. 如果云端失败，记录警告日志
3. 自动查找已配置的本地 Whisper 凭证
4. 使用本地 Whisper 进行回退识别
5. 如果回退也失败，返回详细错误信息

## Whisper 模型文件

模型文件存储路径：`~/Library/Application Support/proxycast/models/whisper/`

下载地址：https://huggingface.co/ggerganov/whisper.cpp/tree/main

| 模型 | 文件名 | 大小 |
|------|--------|------|
| tiny | `ggml-tiny.bin` | ~75MB |
| base | `ggml-base.bin` | ~142MB |
| small | `ggml-small.bin` | ~466MB |
| medium | `ggml-medium.bin` | ~1.5GB |
