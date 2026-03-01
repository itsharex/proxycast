import { useState, useEffect } from "react";
import {
  Eye,
  EyeOff,
  Copy,
  Check,
  RefreshCw,
} from "lucide-react";
import { getConfig, saveConfig, Config } from "@/hooks/useTauri";

export function ProxySettings() {
  const [config, setConfig] = useState<Config | null>(null);
  const [showApiKey, setShowApiKey] = useState(false);
  const [copied, setCopied] = useState(false);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  useEffect(() => {
    loadConfig();
  }, []);

  const loadConfig = async () => {
    try {
      const c = await getConfig();
      setConfig(c);
    } catch (e) {
      console.error(e);
    }
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    setMessage(null);
    try {
      await saveConfig(config);
      setMessage({ type: "success", text: "设置已保存" });
      setTimeout(() => setMessage(null), 3000);
    } catch (e: unknown) {
      const errorMessage = e instanceof Error ? e.message : String(e);
      setMessage({ type: "error", text: `保存失败: ${errorMessage}` });
    }
    setSaving(false);
  };

  const copyApiKey = () => {
    if (config) {
      navigator.clipboard.writeText(config.server.api_key);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (!config) {
    return (
      <div className="flex items-center justify-center h-64">
        <RefreshCw className="h-6 w-6 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-6 max-w-2xl">
      {/* 消息提示 */}
      {message && (
        <div
          className={`rounded-lg border p-3 text-sm ${
            message.type === "error"
              ? "border-destructive bg-destructive/10 text-destructive"
              : "border-green-500 bg-green-50 text-green-700 dark:bg-green-900/20 dark:text-green-400"
          }`}
        >
          {message.text}
        </div>
      )}

      {/* 服务器配置 */}
      <div className="space-y-4">
        <div>
          <h3 className="text-sm font-medium">代理服务配置</h3>
          <p className="text-xs text-muted-foreground">
            配置本地代理服务器参数
          </p>
        </div>

        <div className="space-y-4 p-4 rounded-lg border">
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm font-medium mb-1.5">
                监听地址
              </label>
              <input
                type="text"
                value={config.server.host}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    server: { ...config.server, host: e.target.value },
                  })
                }
                className="w-full px-3 py-2 rounded-lg border bg-background text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
              />
            </div>

            <div>
              <label className="block text-sm font-medium mb-1.5">端口</label>
              <input
                type="number"
                value={config.server.port}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    server: {
                      ...config.server,
                      port: parseInt(e.target.value) || 8999,
                    },
                  })
                }
                className="w-full px-3 py-2 rounded-lg border bg-background text-sm focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
              />
            </div>
          </div>

          <div>
            <label className="block text-sm font-medium mb-1.5">API Key</label>
            <div className="relative">
              <input
                type={showApiKey ? "text" : "password"}
                value={config.server.api_key}
                onChange={(e) =>
                  setConfig({
                    ...config,
                    server: { ...config.server, api_key: e.target.value },
                  })
                }
                className="w-full px-3 py-2 pr-20 rounded-lg border bg-background text-sm font-mono focus:ring-2 focus:ring-primary/20 focus:border-primary outline-none"
              />
              <div className="absolute right-2 top-1/2 flex -translate-y-1/2 gap-1">
                <button
                  type="button"
                  onClick={() => setShowApiKey(!showApiKey)}
                  className="p-1.5 rounded hover:bg-muted"
                  title={showApiKey ? "隐藏" : "显示"}
                >
                  {showApiKey ? (
                    <EyeOff className="h-4 w-4" />
                  ) : (
                    <Eye className="h-4 w-4" />
                  )}
                </button>
                <button
                  type="button"
                  onClick={copyApiKey}
                  className="p-1.5 rounded hover:bg-muted"
                  title="复制"
                >
                  {copied ? (
                    <Check className="h-4 w-4 text-green-500" />
                  ) : (
                    <Copy className="h-4 w-4" />
                  )}
                </button>
              </div>
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              用于验证 API 请求的密钥
            </p>
          </div>

          <button
            onClick={handleSave}
            disabled={saving}
            className="w-full px-4 py-2 rounded-lg bg-primary text-primary-foreground text-sm font-medium hover:bg-primary/90 disabled:opacity-50"
          >
            {saving ? "保存中..." : "保存设置"}
          </button>
        </div>
      </div>

    </div>
  );
}
