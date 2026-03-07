/**
 * @file TextField 表单组件
 * @description 文本输入框
 */

import { useCallback, useEffect, useRef, useState } from "react";
import type { TextFieldComponent, A2UIFormData } from "../../types";
import { resolveDynamicValue } from "../../parser";

interface TextFieldRendererProps {
  component: TextFieldComponent;
  data: Record<string, unknown>;
  formData: A2UIFormData;
  onFormChange: (id: string, value: unknown) => void;
}

export function TextFieldRenderer({
  component,
  data,
  formData,
  onFormChange,
}: TextFieldRendererProps) {
  const label = String(resolveDynamicValue(component.label, data, ""));
  const value =
    (formData[component.id] as string) ??
    String(resolveDynamicValue(component.value, data, ""));
  const isLongText = component.variant === "longText";
  const commitFrameRef = useRef<number | null>(null);
  const latestLocalValueRef = useRef(value);
  const [localValue, setLocalValue] = useState(value);

  useEffect(() => {
    latestLocalValueRef.current = value;
    setLocalValue(value);
  }, [value]);

  const commitValue = useCallback(
    (nextValue: string) => {
      onFormChange(component.id, nextValue);
    },
    [component.id, onFormChange],
  );

  const scheduleCommit = useCallback(
    (nextValue: string) => {
      if (commitFrameRef.current !== null) {
        cancelAnimationFrame(commitFrameRef.current);
      }
      commitFrameRef.current = requestAnimationFrame(() => {
        commitValue(nextValue);
        commitFrameRef.current = null;
      });
    },
    [commitValue],
  );

  const handleInputChange = useCallback(
    (nextValue: string) => {
      latestLocalValueRef.current = nextValue;
      setLocalValue(nextValue);
      scheduleCommit(nextValue);
    },
    [scheduleCommit],
  );

  const handleBlur = useCallback(() => {
    if (commitFrameRef.current !== null) {
      cancelAnimationFrame(commitFrameRef.current);
      commitFrameRef.current = null;
    }
    commitValue(latestLocalValueRef.current);
  }, [commitValue]);

  useEffect(() => {
    return () => {
      if (commitFrameRef.current !== null) {
        cancelAnimationFrame(commitFrameRef.current);
      }
    };
  }, []);

  return (
    <div className="space-y-1.5">
      {label && <label className="text-sm font-medium">{label}</label>}
      {isLongText ? (
        <textarea
          value={localValue}
          onChange={(e) => handleInputChange(e.target.value)}
          onBlur={handleBlur}
          placeholder={component.placeholder}
          className="w-full min-h-[80px] px-3 py-2 text-sm border rounded-md bg-background resize-y"
        />
      ) : (
        <input
          type={
            component.variant === "number"
              ? "number"
              : component.variant === "obscured"
                ? "password"
                : "text"
          }
          value={localValue}
          onChange={(e) => handleInputChange(e.target.value)}
          onBlur={handleBlur}
          placeholder={component.placeholder}
          className="w-full px-3 py-2 text-sm border rounded-md bg-background"
        />
      )}
      {component.helperText && (
        <p className="text-xs text-muted-foreground">{component.helperText}</p>
      )}
    </div>
  );
}
