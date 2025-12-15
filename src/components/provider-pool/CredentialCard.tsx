import {
  Heart,
  HeartOff,
  Trash2,
  RotateCcw,
  Activity,
  Power,
  PowerOff,
  Clock,
  AlertTriangle,
  RefreshCw,
  Key,
  CheckCircle,
  XCircle,
  Database,
  Settings,
} from "lucide-react";
import type { CredentialDisplay } from "@/lib/api/providerPool";

interface CredentialCardProps {
  credential: CredentialDisplay;
  onToggle: () => void;
  onDelete: () => void;
  onReset: () => void;
  onCheckHealth: () => void;
  onRefreshToken?: () => void;
  onEdit: () => void;
  deleting: boolean;
  checkingHealth: boolean;
  refreshingToken?: boolean;
}

export function CredentialCard({
  credential,
  onToggle,
  onDelete,
  onReset,
  onCheckHealth,
  onRefreshToken,
  onEdit,
  deleting,
  checkingHealth,
  refreshingToken,
}: CredentialCardProps) {
  const formatDate = (dateStr?: string) => {
    if (!dateStr) return "从未";
    const date = new Date(dateStr);
    return date.toLocaleString("zh-CN", {
      month: "2-digit",
      day: "2-digit",
      hour: "2-digit",
      minute: "2-digit",
    });
  };

  const getCredentialTypeLabel = (type: string) => {
    const labels: Record<string, string> = {
      kiro_oauth: "OAuth",
      gemini_oauth: "OAuth",
      qwen_oauth: "OAuth",
      openai_key: "API Key",
      claude_key: "API Key",
    };
    return labels[type] || type;
  };

  const isHealthy = credential.is_healthy && !credential.is_disabled;
  const hasError = credential.error_count > 0;
  const isOAuth = credential.credential_type.includes("oauth");

  return (
    <div
      className={`rounded-xl border p-5 transition-all hover:shadow-md ${
        credential.is_disabled
          ? "border-gray-200 bg-gray-50/80 opacity-60 dark:border-gray-700 dark:bg-gray-900/60"
          : isHealthy
            ? "border-green-200 bg-gradient-to-br from-green-50/80 to-green-100/40 dark:border-green-800 dark:bg-gradient-to-br dark:from-green-950/40 dark:to-green-900/20 shadow-green-500/5"
            : "border-red-200 bg-gradient-to-br from-red-50/80 to-red-100/40 dark:border-red-800 dark:bg-gradient-to-br dark:from-red-950/40 dark:to-red-900/20 shadow-red-500/5"
      }`}
    >
      {/* Header */}
      <div className="flex items-start justify-between mb-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-3 mb-2">
            <h4 className="font-semibold text-base truncate">
              {credential.name || `凭证 #${credential.uuid.slice(0, 12)}`}
            </h4>
            <span className="rounded-full bg-muted px-2.5 py-1 text-xs font-medium whitespace-nowrap">
              {getCredentialTypeLabel(credential.credential_type)}
            </span>
          </div>
          <p className="text-xs text-muted-foreground font-mono">
            {credential.uuid.slice(0, 24)}...
          </p>
        </div>
        <div className="flex items-center gap-2">
          <div
            className={`rounded-full p-2 ${
              credential.is_disabled
                ? "bg-gray-100 dark:bg-gray-800"
                : isHealthy
                  ? "bg-green-100 dark:bg-green-900/30"
                  : "bg-red-100 dark:bg-red-900/30"
            }`}
          >
            {credential.is_disabled ? (
              <PowerOff className="h-4 w-4 text-gray-400" />
            ) : isHealthy ? (
              <Heart className="h-4 w-4 text-green-600 dark:text-green-400" />
            ) : (
              <HeartOff className="h-4 w-4 text-red-600 dark:text-red-400" />
            )}
          </div>
        </div>
      </div>

      {/* Stats */}
      <div className="bg-white/50 dark:bg-black/20 rounded-lg p-3 mb-3">
        <div className="grid grid-cols-2 gap-3">
          <div className="flex items-center gap-2">
            <div className="rounded-full bg-blue-100 dark:bg-blue-900/30 p-1.5">
              <Activity className="h-3 w-3 text-blue-600 dark:text-blue-400" />
            </div>
            <div>
              <div className="text-xs text-muted-foreground">使用次数</div>
              <div className="font-semibold text-sm">
                {credential.usage_count}
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <div
              className={`rounded-full p-1.5 ${
                hasError
                  ? "bg-yellow-100 dark:bg-yellow-900/30"
                  : "bg-green-100 dark:bg-green-900/30"
              }`}
            >
              <AlertTriangle
                className={`h-3 w-3 ${
                  hasError
                    ? "text-yellow-600 dark:text-yellow-400"
                    : "text-green-600 dark:text-green-400"
                }`}
              />
            </div>
            <div>
              <div className="text-xs text-muted-foreground">错误次数</div>
              <div className="font-semibold text-sm">
                {credential.error_count}
              </div>
            </div>
          </div>
          <div className="col-span-2 flex items-center gap-2 pt-2 border-t border-border/50">
            <Clock className="h-3 w-3 text-muted-foreground" />
            <span className="text-xs text-muted-foreground">
              最后使用: {formatDate(credential.last_used)}
            </span>
          </div>
        </div>
      </div>

      {/* Health Check Info */}
      {credential.last_health_check_time && (
        <div className="mt-2 text-xs text-muted-foreground">
          <span>
            检查: {formatDate(credential.last_health_check_time)}
            {credential.last_health_check_model &&
              ` (${credential.last_health_check_model})`}
          </span>
        </div>
      )}

      {/* OAuth Status */}
      {isOAuth && credential.oauth_status && (
        <div className="mb-3 rounded-lg border border-blue-200 dark:border-blue-800 bg-blue-50/50 dark:bg-blue-950/30 p-3">
          <div className="flex items-center gap-2 mb-3">
            <div className="rounded-full bg-blue-100 dark:bg-blue-900/30 p-1.5">
              <Key className="h-3 w-3 text-blue-600 dark:text-blue-400" />
            </div>
            <span className="font-semibold text-sm">OAuth 状态</span>
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">
                Access Token
              </span>
              <div className="flex items-center gap-1">
                {credential.oauth_status.has_access_token ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-red-500" />
                )}
                <span className="text-xs font-medium">
                  {credential.oauth_status.has_access_token ? "有效" : "缺失"}
                </span>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">Token 状态</span>
              <div className="flex items-center gap-1">
                {credential.oauth_status.is_token_valid ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-yellow-500" />
                )}
                <span className="text-xs font-medium">
                  {credential.oauth_status.is_token_valid ? "有效" : "需刷新"}
                </span>
              </div>
            </div>
            {credential.oauth_status.expiry_info && (
              <div className="pt-2 border-t border-blue-200/50 dark:border-blue-800/50">
                <span className="text-xs text-muted-foreground">
                  过期时间: {credential.oauth_status.expiry_info}
                </span>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Token Cache Status */}
      {isOAuth && credential.token_cache_status && (
        <div className="mb-3 rounded-lg border border-purple-200 dark:border-purple-800 bg-purple-50/50 dark:bg-purple-950/30 p-3">
          <div className="flex items-center gap-2 mb-3">
            <div className="rounded-full bg-purple-100 dark:bg-purple-900/30 p-1.5">
              <Database className="h-3 w-3 text-purple-600 dark:text-purple-400" />
            </div>
            <span className="font-semibold text-sm">Token 缓存</span>
          </div>
          <div className="space-y-2">
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">缓存状态</span>
              <div className="flex items-center gap-1">
                {credential.token_cache_status.has_cached_token ? (
                  <CheckCircle className="h-4 w-4 text-green-500" />
                ) : (
                  <XCircle className="h-4 w-4 text-gray-400" />
                )}
                <span className="text-xs font-medium">
                  {credential.token_cache_status.has_cached_token
                    ? "已缓存"
                    : "未缓存"}
                </span>
              </div>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-xs text-muted-foreground">有效性</span>
              <div className="flex items-center gap-1">
                {credential.token_cache_status.is_valid ? (
                  credential.token_cache_status.is_expiring_soon ? (
                    <AlertTriangle className="h-4 w-4 text-yellow-500" />
                  ) : (
                    <CheckCircle className="h-4 w-4 text-green-500" />
                  )
                ) : (
                  <XCircle className="h-4 w-4 text-red-500" />
                )}
                <span className="text-xs font-medium">
                  {credential.token_cache_status.is_valid
                    ? credential.token_cache_status.is_expiring_soon
                      ? "即将过期"
                      : "有效"
                    : "已过期"}
                </span>
              </div>
            </div>
            {(credential.token_cache_status.last_refresh ||
              credential.token_cache_status.expiry_time) && (
              <div className="pt-2 border-t border-purple-200/50 dark:border-purple-800/50 space-y-1">
                {credential.token_cache_status.last_refresh && (
                  <div className="text-xs text-muted-foreground">
                    最后刷新:{" "}
                    {formatDate(credential.token_cache_status.last_refresh)}
                  </div>
                )}
                {credential.token_cache_status.expiry_time && (
                  <div className="text-xs text-muted-foreground">
                    过期时间:{" "}
                    {formatDate(credential.token_cache_status.expiry_time)}
                  </div>
                )}
              </div>
            )}
            {credential.token_cache_status.refresh_error_count > 0 && (
              <div className="pt-2 border-t border-red-200/50 dark:border-red-800/50">
                <div className="flex items-center gap-2 text-red-600 dark:text-red-400">
                  <AlertTriangle className="h-3 w-3" />
                  <span className="text-xs font-medium">
                    刷新失败 {credential.token_cache_status.refresh_error_count}{" "}
                    次
                  </span>
                </div>
                {credential.token_cache_status.last_refresh_error && (
                  <div className="mt-1 text-xs text-red-600 dark:text-red-400 truncate">
                    {credential.token_cache_status.last_refresh_error.slice(
                      0,
                      60,
                    )}
                    {credential.token_cache_status.last_refresh_error.length >
                      60 && "..."}
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Error Message */}
      {credential.last_error_message && (
        <div className="mt-2 rounded bg-red-100 p-2 text-xs text-red-700 dark:bg-red-900/30 dark:text-red-300">
          {credential.last_error_message.slice(0, 100)}
          {credential.last_error_message.length > 100 && "..."}
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-2 pt-4 border-t border-border/30">
        <button
          onClick={onToggle}
          className={`flex items-center gap-1 rounded-lg px-3 py-2 text-xs font-medium transition-colors ${
            credential.is_disabled
              ? "bg-green-100 text-green-700 hover:bg-green-200 dark:bg-green-900/30 dark:text-green-400 dark:hover:bg-green-800/40"
              : "bg-gray-100 text-gray-700 hover:bg-gray-200 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
          }`}
          title={credential.is_disabled ? "启用凭证" : "禁用凭证"}
        >
          {credential.is_disabled ? (
            <>
              <Power className="h-3 w-3" />
              启用
            </>
          ) : (
            <>
              <PowerOff className="h-3 w-3" />
              禁用
            </>
          )}
        </button>

        <button
          onClick={onEdit}
          className="flex items-center gap-1 rounded-lg bg-blue-100 px-3 py-2 text-xs font-medium text-blue-700 hover:bg-blue-200 dark:bg-blue-900/30 dark:text-blue-400 dark:hover:bg-blue-800/40 transition-colors"
          title="编辑凭证配置"
        >
          <Settings className="h-3 w-3" />
          编辑
        </button>

        <div className="flex items-center gap-1">
          <button
            onClick={onCheckHealth}
            disabled={checkingHealth}
            className="flex items-center gap-1 rounded-lg bg-emerald-100 px-3 py-2 text-xs font-medium text-emerald-700 hover:bg-emerald-200 disabled:opacity-50 dark:bg-emerald-900/30 dark:text-emerald-400 dark:hover:bg-emerald-800/40 transition-colors"
            title="执行健康检测"
          >
            <Activity
              className={`h-3 w-3 ${checkingHealth ? "animate-pulse" : ""}`}
            />
            检测
          </button>

          {isOAuth && onRefreshToken && (
            <button
              onClick={onRefreshToken}
              disabled={refreshingToken}
              className="flex items-center gap-1 rounded-lg bg-purple-100 px-3 py-2 text-xs font-medium text-purple-700 hover:bg-purple-200 disabled:opacity-50 dark:bg-purple-900/30 dark:text-purple-400 dark:hover:bg-purple-800/40 transition-colors"
              title="刷新 OAuth Token"
            >
              <RefreshCw
                className={`h-3 w-3 ${refreshingToken ? "animate-spin" : ""}`}
              />
              刷新
            </button>
          )}

          <button
            onClick={onReset}
            className="flex items-center gap-1 rounded-lg bg-orange-100 px-3 py-2 text-xs font-medium text-orange-700 hover:bg-orange-200 dark:bg-orange-900/30 dark:text-orange-400 dark:hover:bg-orange-800/40 transition-colors"
            title="重置统计计数器"
          >
            <RotateCcw className="h-3 w-3" />
            重置
          </button>

          <button
            onClick={onDelete}
            disabled={deleting}
            className="flex items-center gap-1 rounded-lg bg-red-100 px-3 py-2 text-xs font-medium text-red-700 hover:bg-red-200 disabled:opacity-50 dark:bg-red-900/30 dark:text-red-400 dark:hover:bg-red-800/40 transition-colors"
            title="删除凭证"
          >
            <Trash2 className="h-3 w-3" />
            删除
          </button>
        </div>
      </div>
    </div>
  );
}
