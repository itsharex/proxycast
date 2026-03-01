import { useEffect, useMemo, useState } from "react";
import { Globe, Image as ImageIcon, RefreshCw } from "lucide-react";
import { open } from "@tauri-apps/plugin-shell";
import { getConfig, saveConfig, type Config } from "@/hooks/useTauri";

type SearchEngine = "google" | "xiaohongshu";
const PEXELS_APPLY_URL = "https://www.pexels.com/api/new/";
const PEXELS_DOC_URL = "https://www.pexels.com/api/";
const PIXABAY_APPLY_URL = "https://pixabay.com/accounts/register/";
const PIXABAY_DOC_URL = "https://pixabay.com/api/docs/";

export function WebSearchSettings() {
  const [config, setConfig] = useState<Config | null>(null);
  const [draftEngine, setDraftEngine] = useState<SearchEngine>("google");
  const [draftPexelsApiKey, setDraftPexelsApiKey] = useState("");
  const [draftPixabayApiKey, setDraftPixabayApiKey] = useState("");
  const [showPexelsApiKey, setShowPexelsApiKey] = useState(false);
  const [showPixabayApiKey, setShowPixabayApiKey] = useState(false);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState<{
    type: "success" | "error";
    text: string;
  } | null>(null);

  const loadConfig = async () => {
    setLoading(true);
    setMessage(null);
    try {
      const nextConfig = await getConfig();
      const engine = (nextConfig.web_search?.engine ||
        "google") as SearchEngine;
      const pexelsApiKey =
        nextConfig.image_gen?.image_search_pexels_api_key || "";
      const pixabayApiKey =
        nextConfig.image_gen?.image_search_pixabay_api_key || "";
      setConfig(nextConfig);
      setDraftEngine(engine);
      setDraftPexelsApiKey(pexelsApiKey);
      setDraftPixabayApiKey(pixabayApiKey);
    } catch (error) {
      console.error("加载网络搜索配置失败:", error);
      setMessage({
        type: "error",
        text: `加载配置失败: ${error instanceof Error ? error.message : String(error)}`,
      });
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    void loadConfig();
  }, []);

  const currentEngine = useMemo(
    () => (config?.web_search?.engine || "google") as SearchEngine,
    [config],
  );
  const currentPexelsApiKey = useMemo(
    () => config?.image_gen?.image_search_pexels_api_key || "",
    [config],
  );
  const currentPixabayApiKey = useMemo(
    () => config?.image_gen?.image_search_pixabay_api_key || "",
    [config],
  );

  const hasUnsavedChanges =
    draftEngine !== currentEngine ||
    draftPexelsApiKey.trim() !== currentPexelsApiKey ||
    draftPixabayApiKey.trim() !== currentPixabayApiKey;
  const pexelsKeyConfigured = draftPexelsApiKey.trim().length > 0;
  const pixabayKeyConfigured = draftPixabayApiKey.trim().length > 0;

  const handleSave = async () => {
    if (!config || !hasUnsavedChanges) return;
    setSaving(true);
    setMessage(null);
    try {
      const nextConfig: Config = {
        ...config,
        web_search: {
          engine: draftEngine,
        },
        image_gen: {
          ...(config.image_gen || {}),
          image_search_pexels_api_key: draftPexelsApiKey.trim(),
          image_search_pixabay_api_key: draftPixabayApiKey.trim(),
        },
      };
      await saveConfig(nextConfig);
      setConfig(nextConfig);
      setMessage({ type: "success", text: "网络搜索设置已保存" });
      setTimeout(() => setMessage(null), 2500);
    } catch (error) {
      setMessage({
        type: "error",
        text: `保存失败: ${error instanceof Error ? error.message : String(error)}`,
      });
    } finally {
      setSaving(false);
    }
  };

  const handleReset = () => {
    setDraftEngine(currentEngine);
    setDraftPexelsApiKey(currentPexelsApiKey);
    setDraftPixabayApiKey(currentPixabayApiKey);
    setMessage(null);
  };

  const openExternalUrl = async (url: string) => {
    try {
      await open(url);
    } catch (error) {
      console.error("打开外部链接失败:", error);
      window.open(url, "_blank");
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-40">
        <RefreshCw className="h-5 w-5 animate-spin text-muted-foreground" />
      </div>
    );
  }

  return (
    <div className="space-y-4 max-w-3xl pb-20">
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

      <div className="rounded-lg border p-5 space-y-4">
        <div className="flex items-center gap-2">
          <Globe className="h-4 w-4 text-primary" />
          <div>
            <h3 className="text-sm font-medium">搜索引擎</h3>
            <p className="text-xs text-muted-foreground">
              选择用于网络搜索的默认搜索引擎。
            </p>
          </div>
        </div>

        <div className="space-y-2">
          <label htmlFor="web-search-engine" className="text-sm font-medium">
            选择搜索引擎
          </label>
          <select
            id="web-search-engine"
            value={draftEngine}
            onChange={(e) => setDraftEngine(e.target.value as SearchEngine)}
            className="w-full h-10 rounded-md border bg-background px-3 text-sm outline-none focus:ring-2 focus:ring-primary/20"
          >
            <option value="google">Google</option>
            <option value="xiaohongshu">小红书</option>
          </select>
          <p className="text-xs text-muted-foreground">
            Google 适用于通用搜索，小红书适用于中文生活方式和购物内容。
          </p>
        </div>
      </div>

      <div className="rounded-lg border p-5 space-y-4">
        <div className="flex items-start justify-between gap-3">
          <div className="flex items-center gap-2">
            <ImageIcon className="h-4 w-4 text-primary" />
            <div>
              <h3 className="text-sm font-medium">联网图片搜索</h3>
              <p className="text-xs text-muted-foreground">
                配置插图页「图片搜索 → 联网搜索」使用的 Pexels API Key。
              </p>
            </div>
          </div>
          <div className="flex flex-col items-end gap-1">
            <span
              className={`inline-flex items-center rounded-full px-2 py-1 text-xs ${
                pexelsKeyConfigured
                  ? "bg-green-500/10 text-green-600 dark:text-green-400"
                  : "bg-muted text-muted-foreground"
              }`}
            >
              Pexels {pexelsKeyConfigured ? "已填写" : "未填写"}
            </span>
            <span
              className={`inline-flex items-center rounded-full px-2 py-1 text-xs ${
                pixabayKeyConfigured
                  ? "bg-green-500/10 text-green-600 dark:text-green-400"
                  : "bg-muted text-muted-foreground"
              }`}
            >
              Pixabay {pixabayKeyConfigured ? "已填写" : "未填写"}
            </span>
          </div>
        </div>

        <div className="space-y-4">
          <label
            htmlFor="web-search-pexels-key"
            className="text-sm font-medium"
          >
            Pexels API Key
          </label>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => void openExternalUrl(PEXELS_APPLY_URL)}
              className="rounded-md border px-3 py-1.5 text-xs hover:bg-muted"
            >
              申请 Pexels Key
            </button>
            <button
              type="button"
              onClick={() => void openExternalUrl(PEXELS_DOC_URL)}
              className="rounded-md border px-3 py-1.5 text-xs hover:bg-muted"
            >
              查看文档
            </button>
          </div>
          <div className="relative">
            <input
              id="web-search-pexels-key"
              type={showPexelsApiKey ? "text" : "password"}
              value={draftPexelsApiKey}
              onChange={(e) => setDraftPexelsApiKey(e.target.value)}
              placeholder="输入 Pexels API Key"
              className="w-full h-10 rounded-md border bg-background px-3 pr-20 text-sm outline-none focus:ring-2 focus:ring-primary/20"
            />
            <button
              type="button"
              onClick={() => setShowPexelsApiKey((prev) => !prev)}
              className="absolute right-1.5 top-1/2 -translate-y-1/2 rounded-md border px-2.5 py-1 text-xs"
            >
              {showPexelsApiKey ? "隐藏" : "显示"}
            </button>
          </div>

          <div className="rounded-md bg-muted/40 px-3 py-2 text-xs text-muted-foreground space-y-1">
            <p>
              未填写时会回退读取环境变量 <code>PEXELS_API_KEY</code>。
            </p>
            <p>申请地址：{PEXELS_APPLY_URL}</p>
            <p>验证路径：插图 → 图片搜索 → 联网搜索。</p>
          </div>

          <div className="h-px bg-border/60" />

          <label
            htmlFor="web-search-pixabay-key"
            className="text-sm font-medium"
          >
            Pixabay API Key
          </label>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => void openExternalUrl(PIXABAY_APPLY_URL)}
              className="rounded-md border px-3 py-1.5 text-xs hover:bg-muted"
            >
              申请 Pixabay Key
            </button>
            <button
              type="button"
              onClick={() => void openExternalUrl(PIXABAY_DOC_URL)}
              className="rounded-md border px-3 py-1.5 text-xs hover:bg-muted"
            >
              查看文档
            </button>
          </div>
          <div className="relative">
            <input
              id="web-search-pixabay-key"
              type={showPixabayApiKey ? "text" : "password"}
              value={draftPixabayApiKey}
              onChange={(e) => setDraftPixabayApiKey(e.target.value)}
              placeholder="输入 Pixabay API Key"
              className="w-full h-10 rounded-md border bg-background px-3 pr-20 text-sm outline-none focus:ring-2 focus:ring-primary/20"
            />
            <button
              type="button"
              onClick={() => setShowPixabayApiKey((prev) => !prev)}
              className="absolute right-1.5 top-1/2 -translate-y-1/2 rounded-md border px-2.5 py-1 text-xs"
            >
              {showPixabayApiKey ? "隐藏" : "显示"}
            </button>
          </div>
          <div className="rounded-md bg-muted/40 px-3 py-2 text-xs text-muted-foreground space-y-1">
            <p>
              未填写时会回退读取环境变量 <code>PIXABAY_API_KEY</code>。
            </p>
            <p>申请地址：{PIXABAY_APPLY_URL}</p>
            <p>验证路径：插图 → 图片搜索 → Pixabay图库。</p>
          </div>
        </div>
      </div>

      <div className="sticky bottom-0 bg-background/95 backdrop-blur supports-[backdrop-filter]:bg-background/80 border rounded-lg px-4 py-3 flex items-center justify-between gap-3">
        <div className="text-sm text-muted-foreground">
          {hasUnsavedChanges ? "未保存的更改" : "所有更改已保存"}
        </div>
        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleReset}
            disabled={!hasUnsavedChanges || saving}
            className="rounded-md border px-3 py-2 text-sm disabled:opacity-50"
          >
            取消
          </button>
          <button
            type="button"
            onClick={handleSave}
            disabled={!hasUnsavedChanges || saving}
            className="rounded-md bg-primary px-4 py-2 text-sm text-primary-foreground hover:bg-primary/90 disabled:opacity-50"
          >
            {saving ? "保存中..." : "保存"}
          </button>
        </div>
      </div>
    </div>
  );
}

export default WebSearchSettings;
