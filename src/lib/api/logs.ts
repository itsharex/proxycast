import { safeInvoke } from "@/lib/dev-bridge";

export interface LogEntry {
  timestamp: string;
  level: string;
  message: string;
}

export async function getLogs(): Promise<LogEntry[]> {
  return safeInvoke("get_logs");
}

export async function getPersistedLogsTail(lines = 200): Promise<LogEntry[]> {
  const safeLines = Number.isFinite(lines)
    ? Math.min(1000, Math.max(20, Math.floor(lines)))
    : 200;
  return safeInvoke("get_persisted_logs_tail", { lines: safeLines });
}

export async function clearLogs(): Promise<void> {
  await safeInvoke("clear_logs");
}

export async function clearDiagnosticLogHistory(): Promise<void> {
  await safeInvoke("clear_diagnostic_log_history");
}
