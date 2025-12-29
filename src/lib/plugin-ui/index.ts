/**
 * @file 插件 UI 系统入口
 * @description 导出插件 UI 系统的所有公共 API
 * @module lib/plugin-ui
 */

// 类型导出
export type {
  // 基础类型
  ComponentId,
  SurfaceId,
  PluginId,
  DataPath,

  // 数据绑定
  BoundValue,
  BoundString,
  BoundNumber,
  BoundBoolean,
  ChildrenDef,
  Action,
  ActionContextItem,

  // 组件类型
  TextVariant,
  ButtonVariant,
  BadgeVariant,
  AlertType,
  Alignment,
  Distribution,
  Direction,
  IconName,

  // 组件定义
  ComponentDef,
  ComponentType,
  ComponentTypeName,
  RowComponent,
  ColumnComponent,
  CardComponent,
  TextComponent,
  IconComponent,
  ButtonComponent,
  BadgeComponent,
  ProgressComponent,
  TextFieldComponent,
  SwitchComponent,
  SelectComponent,
  ListComponent,
  TabsComponent,
  AlertComponent,
  SpinnerComponent,
  EmptyComponent,
  DividerComponent,
  KeyValueComponent,

  // 消息类型
  ServerMessage,
  SurfaceUpdate,
  DataModelUpdate,
  BeginRendering,
  DeleteSurface,
  ClientMessage,
  UserAction,
  ClientError,
  DataEntry,

  // 状态类型
  SurfaceState,
  SurfaceStyles,

  // 渲染器类型
  ComponentRenderer,
  ComponentRendererProps,
  ComponentRegistryEntry,
} from "./types";

// 核心模块导出
export { ComponentRegistry, componentRegistry } from "./ComponentRegistry";
export { SurfaceManager, surfaceManager } from "./SurfaceManager";
export {
  DataStore,
  dataEntriesToObject,
  parsePath,
  getValueByPath,
  setValueByPath,
  mergeDataUpdate,
} from "./DataStore";

// 渲染器导出
export { PluginUIRenderer } from "./PluginUIRenderer";
export { default as PluginUIRendererDefault } from "./PluginUIRenderer";
export { PluginUIContainer } from "./PluginUIContainer";
export { default as PluginUIContainerDefault } from "./PluginUIContainer";

// Hook 导出
export { usePluginUI, usePluginSurface } from "./usePluginUI";

// 组件导出
export {
  standardComponents,
  registerStandardComponents,
  layoutComponents,
  displayComponents,
  inputComponents,
  dataComponents,
} from "./components";

// 初始化函数
import { registerStandardComponents } from "./components";

/**
 * 初始化插件 UI 系统
 * 注册所有标准组件
 */
export function initPluginUI(): void {
  registerStandardComponents();
}
