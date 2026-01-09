import ReactDOM from "react-dom/client";
import { RootRouter } from "./RootRouter";
import "./index.css";

// Initialize Tauri mock for web mode
import "./lib/tauri-mock";

// Initialize i18n configuration
import "./i18n/config";

// 初始化插件组件全局暴露（供动态加载的插件使用）
import "./lib/plugin-components/global";

ReactDOM.createRoot(document.getElementById("root")!).render(<RootRouter />);
