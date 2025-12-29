/**
 * @file 数据展示组件
 * @description List, KeyValue, Alert 等数据展示组件实现
 * @module lib/plugin-ui/components/data
 */

import React from "react";
import { cn } from "@/lib/utils";
import {
  Alert as UIAlert,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import { Info, CheckCircle, AlertTriangle, AlertCircle } from "lucide-react";
import type {
  ComponentRendererProps,
  ListComponent,
  KeyValueComponent,
  AlertComponent,
  AlertType,
  Alignment,
} from "../types";

/**
 * 对齐方式映射
 */
const alignmentMap: Record<Alignment, string> = {
  start: "items-start",
  center: "items-center",
  end: "items-end",
  stretch: "items-stretch",
};

/**
 * List 组件 - 列表容器，支持模板渲染
 */
export const ListRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  renderChildren,
}) => {
  const props = (componentDef.component as ListComponent).List;
  const {
    children,
    direction = "vertical",
    alignment = "stretch",
    gap = 8,
  } = props;

  const isHorizontal = direction === "horizontal";

  return (
    <div
      className={cn(
        "flex",
        isHorizontal ? "flex-row flex-wrap" : "flex-col",
        alignmentMap[alignment],
      )}
      style={{ gap: `${gap}px` }}
    >
      {renderChildren(children)}
    </div>
  );
};

/**
 * KeyValue 组件 - 键值对展示
 */
export const KeyValueRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
}) => {
  const props = (componentDef.component as KeyValueComponent).KeyValue;
  const { items } = props;

  return (
    <div className="space-y-2">
      {items.map((item, index) => {
        const keyText = resolveValue(item.key) || "";
        const valueText = resolveValue(item.value) || "";

        return (
          <div key={index} className="flex justify-between items-center py-1">
            <span className="text-sm text-muted-foreground">{keyText}</span>
            <span className="text-sm font-medium">{valueText}</span>
          </div>
        );
      })}
    </div>
  );
};

/**
 * Alert 图标映射
 */
const alertIconMap: Record<AlertType, React.FC<{ className?: string }>> = {
  info: Info,
  success: CheckCircle,
  warning: AlertTriangle,
  error: AlertCircle,
};

/**
 * Alert 样式映射
 */
const alertStyleMap: Record<AlertType, string> = {
  info: "border-blue-200 bg-blue-50 text-blue-800 [&>svg]:text-blue-500",
  success: "border-green-200 bg-green-50 text-green-800 [&>svg]:text-green-500",
  warning:
    "border-yellow-200 bg-yellow-50 text-yellow-800 [&>svg]:text-yellow-500",
  error: "border-red-200 bg-red-50 text-red-800 [&>svg]:text-red-500",
};

/**
 * Alert 组件
 */
export const AlertRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
}) => {
  const props = (componentDef.component as AlertComponent).Alert;
  const { message, type, title } = props;

  const messageText = resolveValue(message) || "";
  const titleText = title ? resolveValue(title) : undefined;
  const IconComponent = alertIconMap[type];

  return (
    <UIAlert className={alertStyleMap[type]}>
      <IconComponent className="h-4 w-4" />
      {titleText && <AlertTitle>{titleText}</AlertTitle>}
      <AlertDescription>{messageText}</AlertDescription>
    </UIAlert>
  );
};
