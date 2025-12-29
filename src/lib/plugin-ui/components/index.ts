/**
 * @file 组件导出入口
 * @description 导出所有标准组件并注册到组件注册表
 * @module lib/plugin-ui/components
 */

import { componentRegistry as globalRegistry } from "../ComponentRegistry";
import { componentRegistry as standardComponents } from "./registry";

/**
 * 注册所有标准组件到全局注册表
 */
export function registerStandardComponents(): void {
  globalRegistry.registerAll(standardComponents);
}

// 导出各类组件
export {
  layoutComponents,
  displayComponents,
  inputComponents,
  dataComponents,
  componentRegistry as standardComponents,
} from "./registry";

// 导出各个渲染器
export * from "./layout";
export * from "./display";
export * from "./input";
export * from "./data";
