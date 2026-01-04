/**
 * @file 插件组件全局暴露
 * @description 将插件组件库暴露到全局变量，供动态加载的插件使用
 */

import React from "react";
import * as PluginComponents from "./index";

// 将组件库和 React 暴露到全局变量
if (typeof window !== "undefined") {
  (window as unknown as Record<string, unknown>).React = React;
  (window as unknown as Record<string, unknown>).ProxyCastPluginComponents =
    PluginComponents;
}

export {};
