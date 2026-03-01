import type {
  CrashReportingConfig,
  LogEntry,
} from "@/hooks/useTauri";

export interface CrashDiagnosticPayload {
  generated_at: string;
  app_version: string;
  platform: string;
  user_agent: string;
  crash_reporting: CrashReportingConfig;
  frontend_crash_logs: LogEntry[];
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
  appVersion?: string;
  platform: string;
  userAgent: string;
  maxCrashLogs?: number;
}

export function buildCrashDiagnosticPayload(
  params: BuildCrashDiagnosticPayloadParams,
): CrashDiagnosticPayload {
  const {
    crashConfig,
    logs,
    appVersion,
    platform,
    userAgent,
    maxCrashLogs = 30,
  } = params;

  return {
    generated_at: new Date().toISOString(),
    app_version: appVersion || "unknown",
    platform,
    user_agent: userAgent,
    crash_reporting: {
      ...normalizeCrashReportingConfig(crashConfig),
      dsn: maskCrashReportingDsn(crashConfig.dsn),
    },
    frontend_crash_logs: pickFrontendCrashLogs(logs, maxCrashLogs),
  };
}

export async function copyCrashDiagnosticToClipboard(
  payload: CrashDiagnosticPayload,
): Promise<void> {
  await navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
}

export function exportCrashDiagnosticToJson(
  payload: CrashDiagnosticPayload,
): void {
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: "application/json",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `proxycast-crash-diagnostic-${Date.now()}.json`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
