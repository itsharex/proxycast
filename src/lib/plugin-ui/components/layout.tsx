/**
 * @file 布局组件
 * @description Row, Column, Card, Tabs 等布局组件实现
 * @module lib/plugin-ui/components/layout
 */

import React from "react";
import { cn } from "@/lib/utils";
import {
  Card as UICard,
  CardHeader,
  CardTitle,
  CardDescription,
  CardContent,
} from "@/components/ui/card";
import {
  Tabs as UITabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs";
import type {
  ComponentRendererProps,
  RowComponent,
  ColumnComponent,
  CardComponent,
  TabsComponent,
  Distribution,
  Alignment,
} from "../types";

/**
 * 分布方式映射到 CSS justify-content
 */
const distributionMap: Record<Distribution, string> = {
  start: "justify-start",
  center: "justify-center",
  end: "justify-end",
  spaceBetween: "justify-between",
  spaceAround: "justify-around",
  spaceEvenly: "justify-evenly",
};

/**
 * 对齐方式映射到 CSS align-items
 */
const alignmentMap: Record<Alignment, string> = {
  start: "items-start",
  center: "items-center",
  end: "items-end",
  stretch: "items-stretch",
};

/**
 * Row 组件 - 水平布局
 */
export const RowRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  renderChildren,
}) => {
  const props = (componentDef.component as RowComponent).Row;
  const {
    children,
    distribution = "start",
    alignment = "center",
    gap = 8,
  } = props;

  return (
    <div
      className={cn(
        "flex flex-row",
        distributionMap[distribution],
        alignmentMap[alignment],
      )}
      style={{ gap: `${gap}px` }}
    >
      {renderChildren(children)}
    </div>
  );
};

/**
 * Column 组件 - 垂直布局
 */
export const ColumnRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  renderChildren,
}) => {
  const props = (componentDef.component as ColumnComponent).Column;
  const {
    children,
    distribution = "start",
    alignment = "stretch",
    gap = 8,
  } = props;

  return (
    <div
      className={cn(
        "flex flex-col",
        distributionMap[distribution],
        alignmentMap[alignment],
      )}
      style={{ gap: `${gap}px` }}
    >
      {renderChildren(children)}
    </div>
  );
};

/**
 * Card 组件 - 卡片容器
 */
export const CardRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
  renderChild,
}) => {
  const props = (componentDef.component as CardComponent).Card;
  const { child, title, description } = props;

  const titleText = title ? resolveValue(title) : undefined;
  const descText = description ? resolveValue(description) : undefined;

  const hasHeader = titleText || descText;

  return (
    <UICard>
      {hasHeader && (
        <CardHeader>
          {titleText && <CardTitle>{titleText}</CardTitle>}
          {descText && <CardDescription>{descText}</CardDescription>}
        </CardHeader>
      )}
      <CardContent className={hasHeader ? "" : "pt-6"}>
        {renderChild(child)}
      </CardContent>
    </UICard>
  );
};

/**
 * Tabs 组件 - 标签页
 */
export const TabsRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
  renderChild,
}) => {
  const props = (componentDef.component as TabsComponent).Tabs;
  const { items, defaultTab } = props;

  const defaultValue = defaultTab || items[0]?.id;

  return (
    <UITabs defaultValue={defaultValue} className="w-full">
      <TabsList>
        {items.map((item) => {
          const tabTitle = resolveValue(item.title) || item.id;
          return (
            <TabsTrigger key={item.id} value={item.id}>
              {tabTitle}
            </TabsTrigger>
          );
        })}
      </TabsList>
      {items.map((item) => (
        <TabsContent key={item.id} value={item.id}>
          {renderChild(item.child)}
        </TabsContent>
      ))}
    </UITabs>
  );
};
