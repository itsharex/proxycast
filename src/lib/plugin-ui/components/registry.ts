/**
 * @file 组件注册表
 * @description 汇总所有插件 UI 组件的注册表
 * @module lib/plugin-ui/components/registry
 */

import { ListRenderer, KeyValueRenderer, AlertRenderer } from "./data";
import {
  TextRenderer,
  IconRenderer,
  BadgeRenderer,
  ProgressRenderer,
  SpinnerRenderer,
  EmptyRenderer,
  DividerRenderer,
} from "./display";
import {
  ButtonRenderer,
  TextFieldRenderer,
  SwitchRenderer,
  SelectRenderer,
} from "./input";
import {
  RowRenderer,
  ColumnRenderer,
  CardRenderer,
  TabsRenderer,
} from "./layout";

/**
 * 数据展示组件映射
 */
export const dataComponents = {
  List: ListRenderer,
  KeyValue: KeyValueRenderer,
  Alert: AlertRenderer,
};

/**
 * 展示组件映射
 */
export const displayComponents = {
  Text: TextRenderer,
  Icon: IconRenderer,
  Badge: BadgeRenderer,
  Progress: ProgressRenderer,
  Spinner: SpinnerRenderer,
  Empty: EmptyRenderer,
  Divider: DividerRenderer,
};

/**
 * 输入组件映射
 */
export const inputComponents = {
  Button: ButtonRenderer,
  TextField: TextFieldRenderer,
  Switch: SwitchRenderer,
  Select: SelectRenderer,
};

/**
 * 布局组件映射
 */
export const layoutComponents = {
  Row: RowRenderer,
  Column: ColumnRenderer,
  Card: CardRenderer,
  Tabs: TabsRenderer,
};

/**
 * 所有组件注册表
 */
export const componentRegistry = {
  ...dataComponents,
  ...displayComponents,
  ...inputComponents,
  ...layoutComponents,
};
