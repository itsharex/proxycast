/**
 * @file 插件 UI 渲染器
 * @description 核心渲染组件，将声明式 UI 定义渲染为 React 组件
 * @module lib/plugin-ui/PluginUIRenderer
 */

import React, { useCallback, useRef } from "react";
import { componentRegistry } from "./ComponentRegistry";
import { getValueByPath } from "./DataStore";
import type {
  SurfaceState,
  ComponentId,
  ChildrenDef,
  BoundValue,
  UserAction,
  ComponentRendererProps,
} from "./types";

interface PluginUIRendererProps {
  /** Surface 状态 */
  surface: SurfaceState;
  /** 用户操作回调 */
  onAction: (action: UserAction) => void;
  /** 自定义类名 */
  className?: string;
}

/**
 * 插件 UI 渲染器
 * 递归渲染 Surface 中的组件树
 */
export const PluginUIRenderer: React.FC<PluginUIRendererProps> = ({
  surface,
  onAction,
  className,
}) => {
  /**
   * 解析绑定值
   * @param bound - 绑定值定义
   * @param itemData - 列表项数据（用于模板渲染）
   */
  const resolveValue = useCallback(
    <T,>(bound: BoundValue<T>, itemData?: unknown): T | undefined => {
      if (!bound) return undefined;

      // 字面值
      if ("literalString" in bound && !("path" in bound)) {
        return bound.literalString as T;
      }
      if ("literalNumber" in bound && !("path" in bound)) {
        return bound.literalNumber as T;
      }
      if ("literalBoolean" in bound && !("path" in bound)) {
        return bound.literalBoolean as T;
      }
      if ("literalArray" in bound && !("path" in bound)) {
        return bound.literalArray as T;
      }

      // 路径绑定
      if ("path" in bound) {
        const path = bound.path as string;

        // 相对路径（用于模板渲染）
        if (itemData !== undefined && !path.startsWith("/")) {
          if (typeof itemData === "object" && itemData !== null) {
            return (itemData as Record<string, unknown>)[path] as T;
          }
          return itemData as T;
        }

        // 绝对路径
        const value = getValueByPath(surface.dataModel, path);
        if (value !== undefined) {
          return value as T;
        }

        // 如果路径没有值，返回字面值作为默认值
        if ("literalString" in bound) return bound.literalString as T;
        if ("literalNumber" in bound) return bound.literalNumber as T;
        if ("literalBoolean" in bound) return bound.literalBoolean as T;
      }

      return undefined;
    },
    [surface.dataModel],
  );

  /**
   * 渲染子组件列表 - 使用 ref 解决循环依赖
   */
  const renderChildrenRef =
    useRef<(children: ChildrenDef, itemData?: unknown) => React.ReactNode[]>();

  /**
   * 渲染单个子组件
   */
  const renderChild = useCallback(
    (childId: ComponentId, itemData?: unknown): React.ReactNode => {
      const childDef = surface.components.get(childId);
      if (!childDef) {
        console.warn(`[PluginUI] 组件 '${childId}' 未找到`);
        return null;
      }

      return (
        <ComponentRenderer
          key={childId}
          componentDef={childDef}
          surface={surface}
          onAction={onAction}
          resolveValue={(bound) => resolveValue(bound, itemData)}
          renderChild={(id) => renderChild(id, itemData)}
          renderChildren={(children) =>
            renderChildrenRef.current?.(children, itemData) ?? []
          }
          itemData={itemData}
        />
      );
    },
    [surface, onAction, resolveValue],
  );

  /**
   * 渲染子组件列表
   */
  const renderChildren = useCallback(
    (children: ChildrenDef, itemData?: unknown): React.ReactNode[] => {
      // 显式列表
      if (children.explicitList) {
        return children.explicitList.map((childId) =>
          renderChild(childId, itemData),
        );
      }

      // 模板渲染
      if (children.template) {
        const { componentId, dataBinding } = children.template;
        const listData = getValueByPath(surface.dataModel, dataBinding);

        if (!Array.isArray(listData)) {
          console.warn(`[PluginUI] 模板数据绑定 '${dataBinding}' 不是数组`);
          return [];
        }

        return listData.map((item, index) => (
          <React.Fragment key={index}>
            {renderChild(componentId, item)}
          </React.Fragment>
        ));
      }

      return [];
    },
    [surface.dataModel, renderChild],
  );

  // 更新 ref
  renderChildrenRef.current = renderChildren;

  // 如果 Surface 未就绪，显示加载状态
  if (!surface.isReady || !surface.rootId) {
    return (
      <div className={className}>
        <div className="flex items-center justify-center p-4 text-muted-foreground">
          加载中...
        </div>
      </div>
    );
  }

  // 渲染根组件
  return (
    <div className={className} style={getSurfaceStyles(surface)}>
      {renderChild(surface.rootId)}
    </div>
  );
};

/**
 * 获取 Surface 样式
 */
function getSurfaceStyles(surface: SurfaceState): React.CSSProperties {
  const styles: Record<string, string | number> = {};

  if (surface.styles.primaryColor) {
    styles["--primary-color"] = surface.styles.primaryColor;
  }

  if (surface.styles.font) {
    styles["fontFamily"] = surface.styles.font;
  }

  if (surface.styles.borderRadius !== undefined) {
    styles["--border-radius"] = `${surface.styles.borderRadius}px`;
  }

  return styles as React.CSSProperties;
}

/**
 * 单个组件渲染器
 */
interface ComponentRendererInternalProps extends Omit<
  ComponentRendererProps,
  "resolveValue" | "renderChild" | "renderChildren"
> {
  resolveValue: <T>(bound: BoundValue<T>) => T | undefined;
  renderChild: (childId: ComponentId) => React.ReactNode;
  renderChildren: (children: ChildrenDef) => React.ReactNode[];
  itemData?: unknown;
}

const ComponentRenderer: React.FC<ComponentRendererInternalProps> = ({
  componentDef,
  surface,
  onAction,
  resolveValue,
  renderChild,
  renderChildren,
}) => {
  // 获取组件类型
  const componentType = Object.keys(componentDef.component)[0];

  // 从注册表获取渲染器
  const Renderer = componentRegistry.get(componentType);

  if (!Renderer) {
    console.warn(`[PluginUI] 未注册的组件类型: ${componentType}`);
    return (
      <div className="p-2 text-sm text-red-500 bg-red-50 rounded">
        未知组件: {componentType}
      </div>
    );
  }

  // 应用 weight 样式
  const style: React.CSSProperties = {};
  if (componentDef.weight !== undefined) {
    style.flexGrow = componentDef.weight;
  }

  return (
    <div style={style}>
      <Renderer
        componentDef={componentDef}
        surface={surface}
        onAction={onAction}
        resolveValue={resolveValue}
        renderChild={renderChild}
        renderChildren={renderChildren}
      />
    </div>
  );
};

export default PluginUIRenderer;
