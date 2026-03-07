import type {
  CrashReportingConfig,
  LogEntry,
} from "@/hooks/useTauri";
import {
  getInvokeErrorBuffer,
  safeInvoke,
  type InvokeErrorBufferEntry,
} from "@/lib/dev-bridge";
import { getRuntimeAppVersion } from "@/lib/appVersion";
import {
  getFrontendCrashBuffer,
  type FrontendCrashBufferEntry,
} from "@/lib/crashReporting";
import {
  getWorkspaceRepairHistory,
  type WorkspaceRepairRecord,
} from "@/lib/workspaceHealthTelemetry";
import { getActiveContentTarget } from "@/lib/activeContentTarget";
import {
  getThemeWorkbenchDocumentState,
  type ThemeWorkbenchDocumentState,
} from "@/lib/api/project";

export interface CrashDiagnosticPayload {
  generated_at: string;
  app_version: string;
  platform: string;
  user_agent: string;
  locale: string;
  timezone: string;
  page_url: string;
  runtime: "tauri" | "browser";
  crash_reporting: CrashReportingConfig;
  frontend_crash_logs: LogEntry[];
  frontend_crash_buffer?: FrontendCrashBufferEntry[];
  invoke_error_buffer?: InvokeErrorBufferEntry[];
  persisted_log_tail?: LogEntry[];
  workspace_repair_history?: WorkspaceRepairRecord[];
  theme_workbench_document_state?: ThemeWorkbenchDocumentState | null;
  diagnostic_collection_notes?: string[];
}

export type DesktopPlatform = "macos" | "windows" | "linux" | "unknown";

export interface ClipboardPermissionGuide {
  platform: DesktopPlatform;
  title: string;
  steps: string[];
  settingsUrl?: string;
}

export interface CrashDiagnosticExportResult {
  fileName: string;
  locationHint: string;
}

export interface CrashDiagnosticExportOptions {
  sceneTag?: string;
  timestamp?: number;
}

export interface OpenDownloadDirectoryResult {
  openedPath: string;
}

export const DEFAULT_CRASH_REPORTING_CONFIG: CrashReportingConfig = {
  enabled: true,
  dsn: null,
  environment: "production",
  sample_rate: 1,
  send_pii: false,
};

export function normalizeCrashReportingConfig(
  config?: CrashReportingConfig,
): CrashReportingConfig {
  return {
    enabled: config?.enabled ?? DEFAULT_CRASH_REPORTING_CONFIG.enabled,
    dsn: config?.dsn ?? DEFAULT_CRASH_REPORTING_CONFIG.dsn,
    environment:
      config?.environment?.trim() ||
      DEFAULT_CRASH_REPORTING_CONFIG.environment,
    sample_rate:
      typeof config?.sample_rate === "number" &&
      Number.isFinite(config.sample_rate)
        ? Math.min(1, Math.max(0, config.sample_rate))
        : DEFAULT_CRASH_REPORTING_CONFIG.sample_rate,
    send_pii: config?.send_pii ?? DEFAULT_CRASH_REPORTING_CONFIG.send_pii,
  };
}

export function maskCrashReportingDsn(dsn?: string | null): string | null {
  if (!dsn) return null;
  const trimmed = dsn.trim();
  if (!trimmed) return null;
  if (trimmed.length <= 12) return "***";
  return `${trimmed.slice(0, 10)}***${trimmed.slice(-6)}`;
}

export function pickFrontendCrashLogs(
  logs: LogEntry[],
  limit = 30,
): LogEntry[] {
  return logs
    .filter((entry) => entry.message.includes("[FrontendCrash]"))
    .slice(0, limit);
}

interface BuildCrashDiagnosticPayloadParams {
  crashConfig: CrashReportingConfig;
  logs: LogEntry[];
  persistedLogTail?: LogEntry[];
  collectionNotes?: string[];
  appVersion?: string;
  platform: string;
  userAgent: string;
  maxCrashLogs?: number;
  maxInvokeErrors?: number;
  maxPersistedLogs?: number;
  maxWorkspaceRepairs?: number;
  themeWorkbenchDocumentState?: ThemeWorkbenchDocumentState | null;
}

export function buildCrashDiagnosticPayload(
  params: BuildCrashDiagnosticPayloadParams,
): CrashDiagnosticPayload {
  const {
    crashConfig,
    logs,
    persistedLogTail = [],
    collectionNotes = [],
    appVersion,
    platform,
    userAgent,
    maxCrashLogs = 30,
    maxInvokeErrors = 40,
    maxPersistedLogs = 200,
    maxWorkspaceRepairs = 50,
    themeWorkbenchDocumentState = null,
  } = params;

  return {
    generated_at: new Date().toISOString(),
    app_version: getRuntimeAppVersion(appVersion),
    platform,
    user_agent: userAgent,
    locale:
      (typeof navigator !== "undefined" && navigator.language) || "unknown",
    timezone: Intl.DateTimeFormat().resolvedOptions().timeZone || "unknown",
    page_url:
      (typeof window !== "undefined" && window.location?.href) || "unknown",
    runtime: isTauriRuntime() ? "tauri" : "browser",
    crash_reporting: {
      ...normalizeCrashReportingConfig(crashConfig),
      dsn: maskCrashReportingDsn(crashConfig.dsn),
    },
    frontend_crash_logs: pickFrontendCrashLogs(logs, maxCrashLogs),
    frontend_crash_buffer: getFrontendCrashBuffer(maxCrashLogs),
    invoke_error_buffer: getInvokeErrorBuffer(maxInvokeErrors),
    persisted_log_tail: persistedLogTail.slice(-maxPersistedLogs),
    workspace_repair_history: getWorkspaceRepairHistory(maxWorkspaceRepairs),
    theme_workbench_document_state: themeWorkbenchDocumentState,
    diagnostic_collection_notes: collectionNotes.filter((item) =>
      typeof item === "string" && item.trim().length > 0
    ),
  };
}

export async function collectThemeWorkbenchDocumentStateForDiagnostic(): Promise<ThemeWorkbenchDocumentState | null> {
  const activeTarget = getActiveContentTarget();
  if (!activeTarget?.contentId || activeTarget.canvasType !== "document") {
    return null;
  }

  try {
    return await getThemeWorkbenchDocumentState(activeTarget.contentId);
  } catch (error) {
    console.warn("[crashDiagnostic] 获取主题工作台文稿状态失败:", error);
    return null;
  }
}

export async function copyCrashDiagnosticToClipboard(
  payload: CrashDiagnosticPayload,
): Promise<void> {
  const text = buildCrashDiagnosticClipboardText(payload);
  await copyTextToClipboard(text);
}

export async function copyCrashDiagnosticJsonToClipboard(
  payload: CrashDiagnosticPayload,
): Promise<void> {
  const text = JSON.stringify(payload, null, 2);
  await copyTextToClipboard(text);
}

async function copyTextToClipboard(text: string): Promise<void> {
  let lastError: unknown;

  if (typeof navigator !== "undefined" && navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      return;
    } catch (error) {
      lastError = error;
      if (copyTextWithExecCommand(text)) {
        return;
      }
    }
  }

  if (copyTextWithExecCommand(text)) {
    return;
  }

  if (await copyTextViaTauriClipboard(text)) {
    return;
  }

  if (lastError) {
    throw new Error(formatClipboardCopyError(lastError));
  }

  throw new Error("复制诊断信息失败，请重试或使用“导出诊断 JSON”");
}

export function buildCrashDiagnosticClipboardText(
  payload: CrashDiagnosticPayload,
): string {
  const json = JSON.stringify(payload, null, 2);
  const summary = buildDiagnosticSummary(payload);
  return `# ProxyCast 故障诊断请求（可直接给 AI）

请你扮演资深全栈工程师，基于下方诊断数据定位问题并给出可落地修复方案。

## 自动摘要
${summary}

## 你的任务
1. 先判断最可能的根因（按概率排序，至少 3 条）
2. 给出最小改动修复方案（前端/后端分别说明）
3. 提供可直接执行的验证步骤（手工 + 自动化）
4. 标注高风险改动点与回滚方案
5. 如果信息不足，请列出最小补充信息清单

## 输出格式要求
- 根因分析
- 修复方案（含关键代码位置）
- 验证步骤
- 风险与回滚
- 仍需补充的信息

## 诊断数据（JSON）
\`\`\`json
${json}
\`\`\`

## 复现补充（请按需补充）
- 操作路径：
- 期望结果：
- 实际结果：
- 首次出现版本：
- 发生频率：
`;
}

function buildDiagnosticSummary(payload: CrashDiagnosticPayload): string {
  const crashLogCount = payload.frontend_crash_logs.length;
  const localCrashCount = payload.frontend_crash_buffer?.length ?? 0;
  const invokeErrorCount = payload.invoke_error_buffer?.length ?? 0;
  const persistedLogCount = payload.persisted_log_tail?.length ?? 0;
  const workspaceRepairCount = payload.workspace_repair_history?.length ?? 0;
  const versionCount = payload.theme_workbench_document_state?.version_count ?? 0;
  const dsnConfigured = payload.crash_reporting.dsn ? "是" : "否";
  return [
    `- 版本：${payload.app_version}`,
    `- 平台：${payload.platform}（${payload.runtime}）`,
    `- 语言/时区：${payload.locale} / ${payload.timezone}`,
    `- 页面：${payload.page_url}`,
    `- 崩溃日志条数：${crashLogCount}`,
    `- 本地崩溃缓存条数：${localCrashCount}`,
    `- 命令调用失败缓存条数：${invokeErrorCount}`,
    `- 持久化日志尾部行数：${persistedLogCount}`,
    `- Workspace 自动修复记录条数：${workspaceRepairCount}`,
    `- 主题工作台文稿版本数：${versionCount}`,
    `- 崩溃上报已启用：${payload.crash_reporting.enabled ? "是" : "否"}（DSN 已配置：${dsnConfigured}）`,
  ].join("\n");
}

function copyTextWithExecCommand(text: string): boolean {
  if (typeof document === "undefined" || typeof document.execCommand !== "function") {
    return false;
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.left = "-9999px";
  textarea.style.top = "0";
  document.body.appendChild(textarea);
  textarea.focus();
  textarea.select();
  textarea.setSelectionRange(0, text.length);

  try {
    return document.execCommand("copy");
  } catch {
    return false;
  } finally {
    document.body.removeChild(textarea);
  }
}

function formatClipboardCopyError(error: unknown): string {
  if (isClipboardPermissionDeniedError(error)) {
    return "剪贴板权限被系统拒绝，请允许权限后重试，或使用“导出诊断 JSON”";
  }

  const rawMessage = error instanceof Error ? error.message : String(error);
  const normalizedMessage = rawMessage.toLowerCase();

  if (normalizedMessage.includes("document is not focused")) {
    return "当前窗口未激活，先点击应用窗口后重试，或使用“导出诊断 JSON”";
  }

  return "复制诊断信息失败，请重试或使用“导出诊断 JSON”";
}

export function isClipboardPermissionDeniedError(error: unknown): boolean {
  const rawMessage = collectErrorMessages(error);
  const normalizedMessage = rawMessage.toLowerCase();
  return (
    normalizedMessage.includes("not allowed by the user agent") ||
    normalizedMessage.includes("denied permission") ||
    normalizedMessage.includes("notallowederror") ||
    normalizedMessage.includes("权限被系统拒绝") ||
    normalizedMessage.includes("权限拒绝")
  );
}

function collectErrorMessages(error: unknown): string {
  const rootMessage = error instanceof Error ? error.message : String(error);
  if (
    error instanceof Error &&
    "cause" in error &&
    error.cause instanceof Error
  ) {
    return `${rootMessage} ${error.cause.message}`;
  }
  return rootMessage;
}

export function detectDesktopPlatform(
  platform = typeof navigator !== "undefined" ? navigator.platform : "",
  userAgent = typeof navigator !== "undefined" ? navigator.userAgent : "",
): DesktopPlatform {
  const platformText = `${platform} ${userAgent}`.toLowerCase();
  if (platformText.includes("mac")) return "macos";
  if (platformText.includes("win")) return "windows";
  if (platformText.includes("linux") || platformText.includes("x11")) {
    return "linux";
  }
  return "unknown";
}

export function getClipboardPermissionGuide(
  platform = typeof navigator !== "undefined" ? navigator.platform : "",
  userAgent = typeof navigator !== "undefined" ? navigator.userAgent : "",
): ClipboardPermissionGuide {
  const detectedPlatform = detectDesktopPlatform(platform, userAgent);

  if (detectedPlatform === "macos") {
    return {
      platform: detectedPlatform,
      title: "macOS 剪贴板权限指引",
      steps: [
        "先点击 ProxyCast 窗口任意区域，再重试复制。",
        "打开“系统设置 → 隐私与安全性 → 辅助功能”，确认 ProxyCast 已启用。",
        "若仍失败，请使用“导出诊断 JSON”并发送给开发者。",
      ],
      settingsUrl:
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    };
  }

  if (detectedPlatform === "windows") {
    return {
      platform: detectedPlatform,
      title: "Windows 剪贴板权限指引",
      steps: [
        "先点击 ProxyCast 窗口任意区域，再重试复制。",
        "打开“设置 → 隐私和安全性 → 剪贴板”，确认系统剪贴板功能可用。",
        "若企业策略限制剪贴板访问，请改用“导出诊断 JSON”。",
      ],
      settingsUrl: "ms-settings:clipboard",
    };
  }

  if (detectedPlatform === "linux") {
    return {
      platform: detectedPlatform,
      title: "Linux 剪贴板权限指引",
      steps: [
        "先点击 ProxyCast 窗口任意区域，再重试复制。",
        "请在桌面环境隐私设置中确认应用未被限制访问剪贴板。",
        "Wayland 环境若仍失败，建议使用“导出诊断 JSON”。",
      ],
    };
  }

  return {
    platform: detectedPlatform,
    title: "剪贴板权限指引",
    steps: [
      "先点击 ProxyCast 窗口任意区域，再重试复制。",
      "请在系统隐私设置中确认未禁止应用访问剪贴板。",
      "若仍失败，请使用“导出诊断 JSON”。",
    ],
  };
}

export function exportCrashDiagnosticToJson(
  payload: CrashDiagnosticPayload,
  options: CrashDiagnosticExportOptions = {},
): CrashDiagnosticExportResult {
  const fileName = buildCrashDiagnosticFileName(payload, options);
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = fileName;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);

  return {
    fileName,
    locationHint: getDefaultDownloadDirectoryHint(),
  };
}

export function buildCrashDiagnosticFileName(
  payload: CrashDiagnosticPayload,
  options: CrashDiagnosticExportOptions = {},
): string {
  const sceneTag =
    sanitizeDiagnosticSceneTag(options.sceneTag) ||
    inferDiagnosticSceneTag(payload) ||
    "manual-export";
  const appVersionTag =
    sanitizeDiagnosticSceneTag(`v-${payload.app_version || "unknown"}`) ||
    "v-unknown";
  const timestamp = formatDiagnosticTimestamp(options.timestamp ?? Date.now());
  return `proxycast-crash-${sceneTag}-${appVersionTag}-${timestamp}.json`;
}

export async function openCrashDiagnosticDownloadDirectory(): Promise<OpenDownloadDirectoryResult> {
  if (!isTauriRuntime()) {
    throw new Error("当前环境不支持自动打开下载目录，请手动前往系统下载目录");
  }

  const platform = detectDesktopPlatform();
  const homeDir = await safeInvoke<string>("get_home_dir").catch(() => "");
  const candidates = buildDownloadDirectoryCandidates(platform, homeDir);

  for (const path of candidates) {
    try {
      await safeInvoke("reveal_in_finder", { path });
      return { openedPath: path };
    } catch {
      // continue
    }
  }

  throw new Error(`无法自动打开下载目录，请手动前往 ${getDefaultDownloadDirectoryHint()}`);
}

async function copyTextViaTauriClipboard(text: string): Promise<boolean> {
  if (!isTauriRuntime()) {
    return false;
  }

  try {
    const result = await safeInvoke<unknown>("copy_machine_id_to_clipboard", {
      machineId: text,
    });

    if (typeof result === "boolean") {
      return result;
    }
    if (
      typeof result === "object" &&
      result !== null &&
      "success" in result
    ) {
      return Boolean((result as { success?: boolean }).success);
    }
    return Boolean(result);
  } catch {
    return false;
  }
}

function isTauriRuntime(): boolean {
  return Boolean(
    typeof window !== "undefined" &&
      ((window as any).__TAURI__?.core?.invoke || (window as any).__TAURI__?.invoke),
  );
}

function getDefaultDownloadDirectoryHint(): string {
  const platform = detectDesktopPlatform();
  if (platform === "windows") {
    return "%USERPROFILE%/Downloads";
  }
  if (platform === "macos") {
    return "~/Downloads";
  }
  if (platform === "linux") {
    return "~/Downloads";
  }
  return "系统默认下载目录";
}

export function sanitizeDiagnosticSceneTag(
  input?: string | null,
): string | undefined {
  if (!input) return undefined;
  const normalized = input
    .trim()
    .toLowerCase()
    .replace(/[\s_/]+/g, "-")
    .replace(/[^a-z0-9-]/g, "-")
    .replace(/-+/g, "-")
    .replace(/^-|-$/g, "")
    .slice(0, 40);
  return normalized || undefined;
}

function inferDiagnosticSceneTag(payload: CrashDiagnosticPayload): string | undefined {
  const notes = payload.diagnostic_collection_notes ?? [];
  const joined = notes.join(" ").toLowerCase();
  if (joined.includes("workspace") && joined.includes("不存在")) {
    return "workspace-path-missing";
  }
  if (joined.includes("boundary_error")) {
    return "crash-recovery";
  }
  if (joined.includes("clipboard")) {
    return "clipboard-permission";
  }
  return undefined;
}

function formatDiagnosticTimestamp(timestamp: number): string {
  const date = new Date(timestamp);
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hours = String(date.getHours()).padStart(2, "0");
  const minutes = String(date.getMinutes()).padStart(2, "0");
  const seconds = String(date.getSeconds()).padStart(2, "0");
  return `${year}${month}${day}-${hours}${minutes}${seconds}`;
}

function buildDownloadDirectoryCandidates(
  platform: DesktopPlatform,
  homeDirRaw: string,
): string[] {
  const homeDir = homeDirRaw.trim();
  const delimiter = platform === "windows" ? "\\" : "/";
  const candidates: string[] = [];

  const pushCandidate = (value: string) => {
    const normalized = value.trim();
    if (!normalized) return;
    if (!candidates.includes(normalized)) {
      candidates.push(normalized);
    }
  };

  if (homeDir) {
    const hasTrailingDelimiter =
      homeDir.endsWith("/") || homeDir.endsWith("\\");
    const base = hasTrailingDelimiter
      ? homeDir.slice(0, -1)
      : homeDir;
    pushCandidate(`${base}${delimiter}Downloads`);
    if (platform === "windows") {
      pushCandidate(`${base}\\downloads`);
    }
    pushCandidate(base);
  }

  if (platform === "windows") {
    pushCandidate("C:\\Users\\Public\\Downloads");
  }
  if (platform === "linux") {
    pushCandidate("/home");
  }
  if (platform === "macos") {
    pushCandidate("/Users");
  }

  return candidates;
}
