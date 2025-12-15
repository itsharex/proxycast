import { useState, useEffect } from "react";
import {
  Activity,
  Server,
  Zap,
  Clock,
  Play,
  Copy,
  Check,
  ChevronDown,
  ChevronUp,
  Settings,
  RefreshCw,
} from "lucide-react";
import { ModelsTab } from "./ModelsTab";
import { LogsTab } from "./LogsTab";
import { RoutesTab } from "./RoutesTab";
import {
  startServer,
  stopServer,
  getServerStatus,
  getConfig,
  saveConfig,
  reloadCredentials,
  testApi,
  ServerStatus,
  Config,
  TestResult,
  getDefaultProvider,
  setDefaultProvider,
  // OpenAI/Claude Custom
  getOpenAICustomStatus,
  setOpenAICustomConfig,
  getClaudeCustomStatus,
  setClaudeCustomConfig,
  OpenAICustomStatus,
  ClaudeCustomStatus,
} from "@/hooks/useTauri";

interface TestState {
  endpoint: string;
  status: "idle" | "loading" | "success" | "error";
  response?: string;
  time?: number;
  httpStatus?: number;
}

type TabId = "server" | "routes" | "openai" | "claude" | "models" | "logs";

export function ApiServerPage() {
  const [status, setStatus] = useState<ServerStatus | null>(null);
  const [config, setConfig] = useState<Config | null>(null);
  const [loading, setLoading] = useState(false);
  const [_error, setError] = useState<string | null>(null);
  const [testResults, setTestResults] = useState<Record<string, TestState>>({});
  const [copiedCmd, setCopiedCmd] = useState<string | null>(null);
  const [expandedTest, setExpandedTest] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabId>("server");

  // Config editing
  const [editPort, setEditPort] = useState<string>("");
  const [editApiKey, setEditApiKey] = useState<string>("");
  const [defaultProvider, setDefaultProviderState] = useState<string>("kiro");

  // OpenAI Custom state
  const [openaiStatus, setOpenaiStatus] = useState<OpenAICustomStatus | null>(
    null,
  );
  const [openaiApiKey, setOpenaiApiKey] = useState("");
  const [openaiBaseUrl, setOpenaiBaseUrl] = useState("");

  // Claude Custom state
  const [claudeStatus, setClaudeStatus] = useState<ClaudeCustomStatus | null>(
    null,
  );
  const [claudeApiKey, setClaudeApiKey] = useState("");
  const [claudeBaseUrl, setClaudeBaseUrl] = useState("");

  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  const fetchStatus = async () => {
    try {
      const s = await getServerStatus();
      setStatus(s);
    } catch (e) {
      console.error(e);
    }
  };

  const fetchConfig = async () => {
    try {
      const c = await getConfig();
      setConfig(c);
      setEditPort(c.server.port.toString());
      setEditApiKey(c.server.api_key);
    } catch (e) {
      console.error(e);
    }
  };

  const loadOpenAICustomStatus = async () => {
    try {
      const status = await getOpenAICustomStatus();
      setOpenaiStatus(status);
      setOpenaiBaseUrl(status.base_url);
    } catch (e) {
      console.error("Failed to load OpenAI Custom status:", e);
    }
  };

  const loadClaudeCustomStatus = async () => {
    try {
      const status = await getClaudeCustomStatus();
      setClaudeStatus(status);
      setClaudeBaseUrl(status.base_url);
    } catch (e) {
      console.error("Failed to load Claude Custom status:", e);
    }
  };

  useEffect(() => {
    fetchStatus();
    fetchConfig();
    loadDefaultProvider();
    loadOpenAICustomStatus();
    loadClaudeCustomStatus();

    const statusInterval = setInterval(fetchStatus, 3000);
    return () => clearInterval(statusInterval);
  }, []);

  const loadDefaultProvider = async () => {
    try {
      const dp = await getDefaultProvider();
      setDefaultProviderState(dp);
    } catch (e) {
      console.error("Failed to get default provider:", e);
    }
  };

  const handleStart = async () => {
    setLoading(true);
    setError(null);
    try {
      await reloadCredentials();
      await startServer();
      await fetchStatus();
      setMessage({ type: "success", text: "服务已启动" });
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setError(errMsg);
      setMessage({ type: "error", text: `启动失败: ${errMsg}` });
    }
    setLoading(false);
  };

  const handleStop = async () => {
    setLoading(true);
    try {
      await stopServer();
      await fetchStatus();
      setMessage({ type: "success", text: "服务已停止" });
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setError(errMsg);
      setMessage({ type: "error", text: `停止失败: ${errMsg}` });
    }
    setLoading(false);
  };

  const handleSaveServerConfig = async () => {
    if (!config) return;
    setLoading(true);
    try {
      const newConfig = {
        ...config,
        server: {
          ...config.server,
          port: parseInt(editPort) || 3001,
          api_key: editApiKey,
        },
      };
      await saveConfig(newConfig);
      await fetchConfig();
      setMessage({ type: "success", text: "服务器配置已保存" });
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setMessage({ type: "error", text: `保存失败: ${errMsg}` });
    }
    setLoading(false);
  };

  const handleSetDefaultProvider = async (providerId: string) => {
    setLoading(true);
    try {
      await setDefaultProvider(providerId);
      setDefaultProviderState(providerId);
      setMessage({
        type: "success",
        text: `默认 Provider 已切换为: ${providerId}`,
      });
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setMessage({ type: "error", text: `切换失败: ${errMsg}` });
    }
    setLoading(false);
  };

  const handleSaveOpenAIConfig = async () => {
    setLoading(true);
    try {
      await setOpenAICustomConfig(
        openaiApiKey || null,
        openaiBaseUrl || null,
        true,
      );
      await loadOpenAICustomStatus();
      setMessage({ type: "success", text: "OpenAI 配置已保存" });
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setMessage({ type: "error", text: `保存失败: ${errMsg}` });
    }
    setLoading(false);
  };

  const handleSaveClaudeConfig = async () => {
    setLoading(true);
    try {
      await setClaudeCustomConfig(
        claudeApiKey || null,
        claudeBaseUrl || null,
        true,
      );
      await loadClaudeCustomStatus();
      setMessage({ type: "success", text: "Claude 配置已保存" });
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setMessage({ type: "error", text: `保存失败: ${errMsg}` });
    }
    setLoading(false);
  };

  const formatUptime = (secs: number) => {
    const h = Math.floor(secs / 3600);
    const m = Math.floor((secs % 3600) / 60);
    return `${h}h ${m}m`;
  };

  const serverUrl = status
    ? `http://${status.host}:${status.port}`
    : "http://localhost:3001";
  const apiKey = config?.server.api_key || "proxycast-key";

  // Test endpoints
  const testEndpoints = [
    {
      id: "health",
      name: "健康检查",
      method: "GET",
      path: "/health",
      needsAuth: false,
      body: null,
    },
    {
      id: "models",
      name: "模型列表",
      method: "GET",
      path: "/v1/models",
      needsAuth: true,
      body: null,
    },
    {
      id: "chat",
      name: "OpenAI Chat",
      method: "POST",
      path: "/v1/chat/completions",
      needsAuth: true,
      body: JSON.stringify({
        model: "claude-sonnet-4-5",
        messages: [{ role: "user", content: "Say hi in one word" }],
      }),
    },
    {
      id: "anthropic",
      name: "Anthropic Messages",
      method: "POST",
      path: "/v1/messages",
      needsAuth: true,
      body: JSON.stringify({
        model: "claude-sonnet-4-5",
        max_tokens: 100,
        messages: [
          {
            role: "user",
            content: "What is 1+1? Answer with just the number.",
          },
        ],
      }),
    },
  ];

  const runTest = async (endpoint: (typeof testEndpoints)[0]) => {
    setTestResults((prev) => ({
      ...prev,
      [endpoint.id]: { endpoint: endpoint.path, status: "loading" },
    }));

    try {
      const result: TestResult = await testApi(
        endpoint.method,
        endpoint.path,
        endpoint.body,
        endpoint.needsAuth,
      );

      setTestResults((prev) => ({
        ...prev,
        [endpoint.id]: {
          endpoint: endpoint.path,
          status: result.success ? "success" : "error",
          response: result.body || `HTTP ${result.status}: 无响应内容`,
          time: result.time_ms,
          httpStatus: result.status,
        },
      }));
    } catch (e: unknown) {
      const errMsg = e instanceof Error ? e.message : String(e);
      setTestResults((prev) => ({
        ...prev,
        [endpoint.id]: {
          endpoint: endpoint.path,
          status: "error",
          response: `请求失败: ${errMsg}`,
        },
      }));
    }
  };

  const runAllTests = async () => {
    for (const endpoint of testEndpoints) {
      await runTest(endpoint);
    }
  };

  const getCurlCommand = (endpoint: (typeof testEndpoints)[0]) => {
    let cmd = `curl -s ${serverUrl}${endpoint.path}`;
    if (endpoint.needsAuth) {
      cmd += ` \\\n  -H "Authorization: Bearer ${apiKey}"`;
    }
    if (endpoint.body) {
      cmd += ` \\\n  -H "Content-Type: application/json"`;
      cmd += ` \\\n  -d '${endpoint.body}'`;
    }
    return cmd;
  };

  const copyCommand = (id: string, cmd: string) => {
    navigator.clipboard.writeText(cmd);
    setCopiedCmd(id);
    setTimeout(() => setCopiedCmd(null), 2000);
  };

  const getStatusBadge = (result?: TestState) => {
    if (!result || result.status === "idle") {
      return <span className="text-xs text-gray-400">未测试</span>;
    }
    if (result.status === "loading") {
      return <span className="text-xs text-blue-500">测试中...</span>;
    }
    if (result.status === "success") {
      return <span className="text-xs text-green-600">{result.time}ms</span>;
    }
    return (
      <span className="text-xs text-red-500">
        失败 {result.httpStatus ? `(${result.httpStatus})` : ""}
      </span>
    );
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold">API Server</h2>
        <p className="text-muted-foreground">
          管理代理 API 服务器和自定义 API 配置
        </p>
      </div>

      {message && (
        <div
          className={`flex items-center gap-2 rounded-lg border p-3 text-sm ${
            message.type === "success"
              ? "border-green-500 bg-green-50 text-green-700 dark:bg-green-950/30"
              : "border-red-500 bg-red-50 text-red-700 dark:bg-red-950/30"
          }`}
        >
          {message.type === "success" ? (
            <Check className="h-4 w-4" />
          ) : (
            <RefreshCw className="h-4 w-4" />
          )}
          {message.text}
        </div>
      )}

      {/* Status Cards */}
      <div className="grid grid-cols-4 gap-4">
        <div className="rounded-lg border bg-card p-4">
          <div className="flex items-center gap-2">
            <Activity className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">状态</span>
          </div>
          <div className="mt-2 flex items-center gap-2">
            <div
              className={`h-2 w-2 rounded-full ${status?.running ? "bg-green-500" : "bg-red-500"}`}
            />
            <span className="font-medium">
              {status?.running ? "运行中" : "已停止"}
            </span>
          </div>
        </div>

        <div className="rounded-lg border bg-card p-4">
          <div className="flex items-center gap-2">
            <Zap className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">请求数</span>
          </div>
          <div className="mt-2 text-2xl font-bold">{status?.requests || 0}</div>
        </div>

        <div className="rounded-lg border bg-card p-4">
          <div className="flex items-center gap-2">
            <Clock className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">运行时间</span>
          </div>
          <div className="mt-2 font-medium">
            {formatUptime(status?.uptime_secs || 0)}
          </div>
        </div>

        <div className="rounded-lg border bg-card p-4">
          <div className="flex items-center gap-2">
            <Server className="h-4 w-4 text-muted-foreground" />
            <span className="text-sm text-muted-foreground">默认 Provider</span>
          </div>
          <div className="mt-2 font-medium capitalize">{defaultProvider}</div>
        </div>
      </div>

      {/* Tabs */}
      <div className="flex gap-2 border-b overflow-x-auto">
        {[
          { id: "server" as TabId, name: "服务器控制" },
          { id: "routes" as TabId, name: "路由端点" },
          { id: "openai" as TabId, name: "OpenAI 自定义" },
          { id: "claude" as TabId, name: "Claude 自定义" },
          { id: "models" as TabId, name: "可用模型" },
          { id: "logs" as TabId, name: "日志" },
        ].map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`px-4 py-2 text-sm font-medium border-b-2 -mb-px whitespace-nowrap ${
              activeTab === tab.id
                ? "border-primary text-primary"
                : "border-transparent text-muted-foreground hover:text-foreground"
            }`}
          >
            {tab.name}
          </button>
        ))}
      </div>

      {/* Server Control Tab */}
      {activeTab === "server" && (
        <div className="space-y-6">
          {/* Server Control */}
          <div className="rounded-lg border bg-card p-6">
            <h3 className="mb-4 font-semibold flex items-center gap-2">
              <Settings className="h-4 w-4" />
              服务控制
            </h3>
            <div className="flex items-center gap-4 mb-4">
              <button
                className="rounded-lg bg-green-600 px-4 py-2 text-sm font-medium text-white hover:bg-green-700 disabled:opacity-50"
                onClick={handleStart}
                disabled={loading || status?.running}
              >
                {loading ? "处理中..." : "启动服务"}
              </button>
              <button
                className="rounded-lg bg-red-600 px-4 py-2 text-sm font-medium text-white hover:bg-red-700 disabled:opacity-50"
                onClick={handleStop}
                disabled={loading || !status?.running}
              >
                停止服务
              </button>
            </div>

            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <label className="block text-muted-foreground mb-1">端口</label>
                <input
                  type="number"
                  value={editPort}
                  onChange={(e) => setEditPort(e.target.value)}
                  className="w-full rounded-lg border bg-background px-3 py-2"
                />
              </div>
              <div>
                <label className="block text-muted-foreground mb-1">
                  API Key
                </label>
                <input
                  type="text"
                  value={editApiKey}
                  onChange={(e) => setEditApiKey(e.target.value)}
                  className="w-full rounded-lg border bg-background px-3 py-2"
                />
              </div>
            </div>

            <div className="mt-4 flex items-center justify-between">
              <div className="text-sm text-muted-foreground">
                API 地址:{" "}
                <code className="rounded bg-muted px-2 py-1">{serverUrl}</code>
              </div>
              <button
                onClick={handleSaveServerConfig}
                disabled={loading}
                className="rounded-lg border px-4 py-2 text-sm font-medium hover:bg-muted disabled:opacity-50"
              >
                保存配置
              </button>
            </div>
          </div>

          {/* Default Provider */}
          <div className="rounded-lg border bg-card p-6">
            <h3 className="mb-4 font-semibold">默认 Provider</h3>
            <div className="flex flex-wrap gap-2">
              {["kiro", "gemini", "qwen", "openai", "claude"].map((p) => (
                <button
                  key={p}
                  onClick={() => handleSetDefaultProvider(p)}
                  disabled={loading}
                  className={`rounded-lg px-4 py-2 text-sm font-medium ${
                    defaultProvider === p
                      ? "bg-primary text-primary-foreground"
                      : "border hover:bg-muted"
                  } disabled:opacity-50`}
                >
                  {p === "kiro"
                    ? "Kiro Claude"
                    : p === "gemini"
                      ? "Gemini CLI"
                      : p === "qwen"
                        ? "通义千问"
                        : p === "openai"
                          ? "OpenAI 自定义"
                          : "Claude 自定义"}
                </button>
              ))}
            </div>
          </div>

          {/* API Testing */}
          <div className="rounded-lg border bg-card p-6">
            <div className="mb-4 flex items-center justify-between">
              <h3 className="font-semibold">API 测试</h3>
              <button
                onClick={runAllTests}
                disabled={!status?.running}
                className="flex items-center gap-2 rounded-lg bg-primary px-3 py-1.5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
              >
                <Play className="h-4 w-4" />
                测试全部
              </button>
            </div>

            <div className="space-y-3">
              {testEndpoints.map((endpoint) => {
                const result = testResults[endpoint.id];
                const isExpanded = expandedTest === endpoint.id;
                const curlCmd = getCurlCommand(endpoint);

                return (
                  <div
                    key={endpoint.id}
                    className="rounded-lg border bg-background"
                  >
                    <div className="flex items-center justify-between p-3">
                      <div className="flex items-center gap-3">
                        <span
                          className={`rounded px-2 py-0.5 text-xs font-medium ${
                            endpoint.method === "GET"
                              ? "bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400"
                              : "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400"
                          }`}
                        >
                          {endpoint.method}
                        </span>
                        <span className="font-medium">{endpoint.name}</span>
                        <code className="text-xs text-muted-foreground">
                          {endpoint.path}
                        </code>
                        {getStatusBadge(result)}
                      </div>
                      <div className="flex items-center gap-2">
                        <button
                          onClick={() => copyCommand(endpoint.id, curlCmd)}
                          className="rounded p-1.5 hover:bg-muted"
                          title="复制 curl 命令"
                        >
                          {copiedCmd === endpoint.id ? (
                            <Check className="h-4 w-4 text-green-500" />
                          ) : (
                            <Copy className="h-4 w-4" />
                          )}
                        </button>
                        <button
                          onClick={() => runTest(endpoint)}
                          disabled={
                            !status?.running || result?.status === "loading"
                          }
                          className="rounded bg-primary/10 px-2 py-1 text-xs font-medium text-primary hover:bg-primary/20 disabled:opacity-50"
                        >
                          测试
                        </button>
                        <button
                          onClick={() =>
                            setExpandedTest(isExpanded ? null : endpoint.id)
                          }
                          className="rounded p-1.5 hover:bg-muted"
                        >
                          {isExpanded ? (
                            <ChevronUp className="h-4 w-4" />
                          ) : (
                            <ChevronDown className="h-4 w-4" />
                          )}
                        </button>
                      </div>
                    </div>

                    {isExpanded && (
                      <div className="border-t p-3 space-y-3">
                        <div>
                          <p className="mb-1 text-xs font-medium text-muted-foreground">
                            curl 命令
                          </p>
                          <pre className="rounded bg-muted p-2 text-xs overflow-x-auto">
                            {curlCmd}
                          </pre>
                        </div>
                        {result?.response && (
                          <div>
                            <p className="mb-1 text-xs font-medium text-muted-foreground">
                              响应{" "}
                              {result.httpStatus &&
                                `(HTTP ${result.httpStatus})`}
                            </p>
                            <pre
                              className={`rounded p-2 text-xs overflow-x-auto max-h-40 ${
                                result.status === "success"
                                  ? "bg-green-50 dark:bg-green-950/30"
                                  : "bg-red-50 dark:bg-red-950/30"
                              }`}
                            >
                              {(() => {
                                try {
                                  return JSON.stringify(
                                    JSON.parse(result.response),
                                    null,
                                    2,
                                  );
                                } catch {
                                  return result.response || "(空响应)";
                                }
                              })()}
                            </pre>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      )}

      {/* Routes Tab */}
      {activeTab === "routes" && <RoutesTab />}

      {/* OpenAI Custom Tab */}
      {activeTab === "openai" && (
        <div className="rounded-lg border bg-card p-6">
          <h3 className="mb-4 font-semibold">OpenAI 自定义 API 配置</h3>
          <p className="text-sm text-muted-foreground mb-4">
            配置自定义 OpenAI 兼容 API 端点，用于转发请求到其他 OpenAI 兼容服务
          </p>
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-muted-foreground mb-1">
                API Key
              </label>
              <input
                type="password"
                value={openaiApiKey}
                onChange={(e) => setOpenaiApiKey(e.target.value)}
                placeholder="sk-..."
                className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">
                Base URL
              </label>
              <input
                type="text"
                value={openaiBaseUrl}
                onChange={(e) => setOpenaiBaseUrl(e.target.value)}
                placeholder="https://api.openai.com/v1"
                className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
              />
            </div>
            <div className="flex items-center gap-2 text-sm">
              <span className="text-muted-foreground">状态:</span>
              <span
                className={
                  openaiStatus?.has_api_key ? "text-green-600" : "text-red-500"
                }
              >
                {openaiStatus?.has_api_key ? "已配置" : "未配置"}
              </span>
            </div>
            <button
              onClick={handleSaveOpenAIConfig}
              disabled={loading}
              className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {loading ? "保存中..." : "保存配置"}
            </button>
          </div>
        </div>
      )}

      {/* Claude Custom Tab */}
      {activeTab === "claude" && (
        <div className="rounded-lg border bg-card p-6">
          <h3 className="mb-4 font-semibold">Claude 自定义 API 配置</h3>
          <p className="text-sm text-muted-foreground mb-4">
            配置自定义 Claude API 端点，用于直接调用 Anthropic API
          </p>
          <div className="space-y-4">
            <div>
              <label className="block text-sm text-muted-foreground mb-1">
                API Key
              </label>
              <input
                type="password"
                value={claudeApiKey}
                onChange={(e) => setClaudeApiKey(e.target.value)}
                placeholder="sk-ant-..."
                className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
              />
            </div>
            <div>
              <label className="block text-sm text-muted-foreground mb-1">
                Base URL
              </label>
              <input
                type="text"
                value={claudeBaseUrl}
                onChange={(e) => setClaudeBaseUrl(e.target.value)}
                placeholder="https://api.anthropic.com"
                className="w-full rounded-lg border bg-background px-3 py-2 text-sm"
              />
            </div>
            <div className="flex items-center gap-2 text-sm">
              <span className="text-muted-foreground">状态:</span>
              <span
                className={
                  claudeStatus?.has_api_key ? "text-green-600" : "text-red-500"
                }
              >
                {claudeStatus?.has_api_key ? "已配置" : "未配置"}
              </span>
            </div>
            <button
              onClick={handleSaveClaudeConfig}
              disabled={loading}
              className="flex items-center gap-2 rounded-lg bg-primary px-4 py-2 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
            >
              {loading ? "保存中..." : "保存配置"}
            </button>
          </div>
        </div>
      )}

      {/* Models Tab */}
      {activeTab === "models" && <ModelsTab />}

      {/* Logs Tab */}
      {activeTab === "logs" && <LogsTab />}
    </div>
  );
}
