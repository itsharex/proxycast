/**
 * @file 展示组件
 * @description Text, Icon, Badge, Progress 等展示组件实现
 * @module lib/plugin-ui/components/display
 */

import React from "react";
import { cn } from "@/lib/utils";
import { Badge as UIBadge } from "@/components/ui/badge";
import { Progress as UIProgress } from "@/components/ui/progress";
import {
  Plus,
  Check,
  X,
  Trash2,
  Edit,
  RefreshCw,
  Search,
  Settings,
  Info,
  AlertTriangle,
  AlertCircle,
  ChevronDown,
  ChevronUp,
  ChevronLeft,
  ChevronRight,
  Power,
  PowerOff,
  Folder,
  File,
  Copy,
  Play,
  Pause,
  Square,
  Download,
  Upload,
  Loader2,
  type LucideIcon,
} from "lucide-react";
import type {
  ComponentRendererProps,
  TextComponent,
  IconComponent,
  BadgeComponent,
  ProgressComponent,
  SpinnerComponent,
  EmptyComponent,
  DividerComponent,
  TextVariant,
  BadgeVariant,
  IconName,
} from "../types";

/**
 * 文本变体样式映射
 */
const textVariantStyles: Record<TextVariant, string> = {
  h1: "text-4xl font-bold",
  h2: "text-3xl font-semibold",
  h3: "text-2xl font-semibold",
  h4: "text-xl font-medium",
  h5: "text-lg font-medium",
  body: "text-base",
  caption: "text-sm text-muted-foreground",
};

/**
 * Text 组件
 */
export const TextRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
}) => {
  const props = (componentDef.component as TextComponent).Text;
  const { text, variant = "body" } = props;

  const textContent = resolveValue(text) || "";

  return <span className={textVariantStyles[variant]}>{textContent}</span>;
};

/**
 * 图标映射
 */
const iconMap: Record<IconName, LucideIcon> = {
  add: Plus,
  check: Check,
  close: X,
  delete: Trash2,
  edit: Edit,
  refresh: RefreshCw,
  search: Search,
  settings: Settings,
  info: Info,
  warning: AlertTriangle,
  error: AlertCircle,
  chevronDown: ChevronDown,
  chevronUp: ChevronUp,
  chevronLeft: ChevronLeft,
  chevronRight: ChevronRight,
  power: Power,
  powerOff: PowerOff,
  folder: Folder,
  file: File,
  copy: Copy,
  play: Play,
  pause: Pause,
  stop: Square,
  download: Download,
  upload: Upload,
};

/**
 * Icon 组件
 */
export const IconRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
}) => {
  const props = (componentDef.component as IconComponent).Icon;
  const { name, size = 16, color } = props;

  const iconName = resolveValue(name) as IconName;
  const IconComponent = iconMap[iconName];

  if (!IconComponent) {
    console.warn(`[PluginUI] 未知图标: ${iconName}`);
    return null;
  }

  return (
    <IconComponent className={cn("inline-block")} size={size} color={color} />
  );
};

/**
 * Badge 变体样式映射
 */
const badgeVariantMap: Record<
  BadgeVariant,
  "default" | "secondary" | "destructive" | "outline"
> = {
  default: "default",
  success: "default",
  warning: "secondary",
  error: "destructive",
  info: "outline",
};

const badgeColorMap: Record<BadgeVariant, string> = {
  default: "",
  success: "bg-green-100 text-green-800 hover:bg-green-100",
  warning: "bg-yellow-100 text-yellow-800 hover:bg-yellow-100",
  error: "",
  info: "bg-blue-100 text-blue-800 hover:bg-blue-100",
};

/**
 * Badge 组件
 */
export const BadgeRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
}) => {
  const props = (componentDef.component as BadgeComponent).Badge;
  const { text, variant } = props;

  const textContent = resolveValue(text) || "";
  const variantValue = (
    variant ? resolveValue(variant) : "default"
  ) as BadgeVariant;

  return (
    <UIBadge
      variant={badgeVariantMap[variantValue]}
      className={badgeColorMap[variantValue]}
    >
      {textContent}
    </UIBadge>
  );
};

/**
 * Progress 组件
 */
export const ProgressRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
}) => {
  const props = (componentDef.component as ProgressComponent).Progress;
  const { value, max = 100 } = props;

  const numValue = resolveValue(value) || 0;
  const percentage = (numValue / max) * 100;

  return <UIProgress value={percentage} className="w-full" />;
};

/**
 * Spinner 组件
 */
export const SpinnerRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
}) => {
  const props = (componentDef.component as SpinnerComponent).Spinner;
  const { size = 24 } = props;

  return (
    <Loader2
      className="animate-spin text-muted-foreground"
      style={{ width: size, height: size }}
    />
  );
};

/**
 * Empty 组件
 */
export const EmptyRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
  resolveValue,
}) => {
  const props = (componentDef.component as EmptyComponent).Empty;
  const { icon, title, description } = props;

  const IconComponent = icon ? iconMap[icon] : null;
  const titleText = title ? resolveValue(title) : undefined;
  const descText = description ? resolveValue(description) : undefined;

  return (
    <div className="flex flex-col items-center justify-center py-8 text-center">
      {IconComponent && (
        <IconComponent className="h-12 w-12 text-muted-foreground/50 mb-4" />
      )}
      {titleText && (
        <p className="text-lg font-medium text-muted-foreground">{titleText}</p>
      )}
      {descText && (
        <p className="text-sm text-muted-foreground/70 mt-1">{descText}</p>
      )}
    </div>
  );
};

/**
 * Divider 组件
 */
export const DividerRenderer: React.FC<ComponentRendererProps> = ({
  componentDef,
}) => {
  const props = (componentDef.component as DividerComponent).Divider;
  const { axis = "horizontal" } = props;

  if (axis === "vertical") {
    return <div className="w-px h-full bg-border" />;
  }

  return <div className="h-px w-full bg-border" />;
};
