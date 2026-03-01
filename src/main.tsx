import ReactDOM from "react-dom/client";
import { RootRouter } from "./RootRouter";
import "./index.css";

// Initialize Tauri mock for web mode
import "./lib/tauri-mock/index";

// Initialize i18n configuration
import "./i18n/config";
import { initCrashReporting } from "@/lib/crashReporting";

// 注册 Artifact 轻量渲染器
import { registerLightweightRenderers } from "./components/artifact";
registerLightweightRenderers();

void initCrashReporting();

ReactDOM.createRoot(document.getElementById("root")!).render(<RootRouter />);
