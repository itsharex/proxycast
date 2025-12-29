/**
 * @file 输入组件
 * @description Button, TextField, Switch, Select 等输入组件实现
 * @module lib/plugin-ui/components/input
 */

import React, { useCallback } from "react";
import { Button as UIButton } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Switch as UISwitch } from "@/components/ui/switch";
import {
  Select as UISelect,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import type {
  ComponentRendererProps,
  ButtonComponent,
  TextFieldComponent,
  SwitchComponent,
  SelectComponent,
  UserAction,
  ButtonVariant,
} from "../types";

/**
 * 按钮变体映射
 */
const buttonVariantMap: Record<
  ButtonVariant,
  "default" | "destructive" | "outline" | "secondary" | "ghost"
> = {
  default: "default",
  primary: "default",
  secondary: "secondary",
  destructive: "destructive",
  outline: "outline",
  ghost: "ghost",
};

/**
 * Button 组件
 */
export const ButtonRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  surface,
  onAction,
  resolveValue,
  renderChild,
}) => {
  const props = (componentDef.component as ButtonComponent).Button;
  const { child, action, variant = "default", disabled } = props;

  const isDisabled = disabled ? resolveValue(disabled) : false;

  const handleClick = useCallback(() => {
    // 解析 action context
    const context: Record<string, unknown> = {};
    if (action.context) {
      for (const item of action.context) {
        context[item.key] = resolveValue(item.value);
      }
    }

    const userAction: UserAction = {
      name: action.name,
      surfaceId: surface.surfaceId,
      sourceComponentId: componentDef.id,
      context,
      timestamp: new Date().toISOString(),
    };

    onAction(userAction);
  }, [action, surface.surfaceId, componentDef.id, onAction, resolveValue]);

  return (
    <UIButton
      variant={buttonVariantMap[variant]}
      disabled={isDisabled}
      onClick={handleClick}
    >
      {renderChild(child)}
    </UIButton>
  );
};

/**
 * TextField 组件
 */
export const TextFieldRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  surface,
  onAction,
  resolveValue,
}) => {
  const props = (componentDef.component as TextFieldComponent).TextField;
  const { label, value, placeholder, type = "text", disabled } = props;

  const labelText = resolveValue(label) || "";
  const valueText = value ? resolveValue(value) || "" : "";
  const placeholderText = placeholder ? resolveValue(placeholder) : undefined;
  const isDisabled = disabled ? resolveValue(disabled) : false;

  // 获取绑定路径用于更新
  const valuePath = value && "path" in value ? value.path : null;

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      if (!valuePath) return;

      const userAction: UserAction = {
        name: "__data_update__",
        surfaceId: surface.surfaceId,
        sourceComponentId: componentDef.id,
        context: {
          path: valuePath,
          value: e.target.value,
        },
        timestamp: new Date().toISOString(),
      };

      onAction(userAction);
    },
    [valuePath, surface.surfaceId, componentDef.id, onAction],
  );

  const inputId = `input-${componentDef.id}`;

  return (
    <div className="space-y-2">
      <Label htmlFor={inputId}>{labelText}</Label>
      <Input
        id={inputId}
        type={type}
        value={valueText}
        placeholder={placeholderText}
        disabled={isDisabled}
        onChange={handleChange}
      />
    </div>
  );
};

/**
 * Switch 组件
 */
export const SwitchRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  surface,
  onAction,
  resolveValue,
}) => {
  const props = (componentDef.component as SwitchComponent).Switch;
  const { label, checked, disabled } = props;

  const labelText = resolveValue(label) || "";
  const isChecked = resolveValue(checked) || false;
  const isDisabled = disabled ? resolveValue(disabled) : false;

  // 获取绑定路径用于更新
  const checkedPath = "path" in checked ? checked.path : null;

  const handleChange = useCallback(
    (newChecked: boolean) => {
      if (!checkedPath) return;

      const userAction: UserAction = {
        name: "__data_update__",
        surfaceId: surface.surfaceId,
        sourceComponentId: componentDef.id,
        context: {
          path: checkedPath,
          value: newChecked,
        },
        timestamp: new Date().toISOString(),
      };

      onAction(userAction);
    },
    [checkedPath, surface.surfaceId, componentDef.id, onAction],
  );

  const switchId = `switch-${componentDef.id}`;

  return (
    <div className="flex items-center space-x-2">
      <UISwitch
        id={switchId}
        checked={isChecked}
        disabled={isDisabled}
        onCheckedChange={handleChange}
      />
      <Label htmlFor={switchId}>{labelText}</Label>
    </div>
  );
};

/**
 * Select 组件
 */
export const SelectRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  surface,
  onAction,
  resolveValue,
}) => {
  const props = (componentDef.component as SelectComponent).Select;
  const { label, value, options, disabled } = props;

  const labelText = label ? resolveValue(label) : undefined;
  const selectedValue = resolveValue(value) || "";
  const isDisabled = disabled ? resolveValue(disabled) : false;

  // 获取绑定路径用于更新
  const valuePath = "path" in value ? value.path : null;

  const handleChange = useCallback(
    (newValue: string) => {
      if (!valuePath) return;

      const userAction: UserAction = {
        name: "__data_update__",
        surfaceId: surface.surfaceId,
        sourceComponentId: componentDef.id,
        context: {
          path: valuePath,
          value: newValue,
        },
        timestamp: new Date().toISOString(),
      };

      onAction(userAction);
    },
    [valuePath, surface.surfaceId, componentDef.id, onAction],
  );

  return (
    <div className="space-y-2">
      {labelText && <Label>{labelText}</Label>}
      <UISelect
        value={selectedValue}
        disabled={isDisabled}
        onValueChange={handleChange}
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map((option, index) => {
            const optionLabel = resolveValue(option.label) || option.value;
            return (
              <SelectItem key={index} value={option.value}>
                {optionLabel}
              </SelectItem>
            );
          })}
        </SelectContent>
      </UISelect>
    </div>
  );
};
