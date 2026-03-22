/**
 * @file 元素工具栏组件
 * @description 底部工具栏，提供添加文字、图片、形状、背景、对齐等功能
 * @module components/content-creator/canvas/poster/ElementToolbar
 */

import React, { memo, useState } from "react";
import styled from "styled-components";
import {
  MousePointer2,
  Image,
  Square,
  Type,
  Palette,
  Circle,
  Triangle,
  Minus,
  AlignCenterHorizontal,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import type { ElementToolbarProps, ShapeType } from "./types";
import { AlignmentToolbar } from "./AlignmentToolbar";

const ToolbarContainer = styled.div`
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 8px 12px;
  background: hsl(var(--background));
  border-top: 1px solid hsl(var(--border));
  gap: 4px;
`;

const ToolButton = styled(Button)<{ $active?: boolean }>`
  height: 40px;
  width: 40px;
  ${(props) =>
    props.$active &&
    `
    background: hsl(var(--accent));
    color: hsl(var(--accent-foreground));
  `}
`;

const ShapeGrid = styled.div`
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 4px;
  padding: 4px;
`;

const ShapeButton = styled(Button)`
  height: 48px;
  width: 48px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 2px;
  font-size: 10px;
`;

/**
 * 工具类型
 */
type ToolType = "select" | "text" | "image" | "shape" | "background";

/**
 * 元素工具栏组件
 */
export const ElementToolbar: React.FC<ElementToolbarProps> = memo(
  ({
    onAddText,
    onAddImage,
    onAddShape,
    onSetBackground,
    hasSelection = false,
    gridSnapEnabled = false,
    onAlign,
    onToggleGridSnap,
  }) => {
    const [activeTool, setActiveTool] = useState<ToolType>("select");
    const [shapePopoverOpen, setShapePopoverOpen] = useState(false);
    const [alignPopoverOpen, setAlignPopoverOpen] = useState(false);

    const handleToolClick = (tool: ToolType) => {
      setActiveTool(tool);

      switch (tool) {
        case "text":
          onAddText();
          setActiveTool("select");
          break;
        case "image":
          void onAddImage();
          setActiveTool("select");
          break;
        case "background":
          onSetBackground();
          break;
      }
    };

    const handleShapeSelect = (type: ShapeType) => {
      onAddShape(type);
      setShapePopoverOpen(false);
      setActiveTool("select");
    };

    return (
      <TooltipProvider>
        <ToolbarContainer>
          <Tooltip>
            <TooltipTrigger asChild>
              <ToolButton
                variant="ghost"
                size="icon"
                $active={activeTool === "select"}
                onClick={() => setActiveTool("select")}
              >
                <MousePointer2 className="h-5 w-5" />
              </ToolButton>
            </TooltipTrigger>
            <TooltipContent>选择工具 (V)</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <ToolButton
                variant="ghost"
                size="icon"
                onClick={() => handleToolClick("text")}
              >
                <Type className="h-5 w-5" />
              </ToolButton>
            </TooltipTrigger>
            <TooltipContent>添加文字 (T)</TooltipContent>
          </Tooltip>

          <Tooltip>
            <TooltipTrigger asChild>
              <ToolButton
                variant="ghost"
                size="icon"
                onClick={() => handleToolClick("image")}
              >
                <Image className="h-5 w-5" />
              </ToolButton>
            </TooltipTrigger>
            <TooltipContent>添加图片</TooltipContent>
          </Tooltip>

          <Popover open={shapePopoverOpen} onOpenChange={setShapePopoverOpen}>
            <Tooltip>
              <TooltipTrigger asChild>
                <PopoverTrigger asChild>
                  <ToolButton
                    variant="ghost"
                    size="icon"
                    $active={shapePopoverOpen}
                  >
                    <Square className="h-5 w-5" />
                  </ToolButton>
                </PopoverTrigger>
              </TooltipTrigger>
              <TooltipContent>添加形状</TooltipContent>
            </Tooltip>
            <PopoverContent className="w-auto p-2" side="top">
              <ShapeGrid>
                <ShapeButton
                  variant="ghost"
                  onClick={() => handleShapeSelect("rect")}
                >
                  <Square className="h-5 w-5" />
                  <span>矩形</span>
                </ShapeButton>
                <ShapeButton
                  variant="ghost"
                  onClick={() => handleShapeSelect("circle")}
                >
                  <Circle className="h-5 w-5" />
                  <span>圆形</span>
                </ShapeButton>
                <ShapeButton
                  variant="ghost"
                  onClick={() => handleShapeSelect("triangle")}
                >
                  <Triangle className="h-5 w-5" />
                  <span>三角形</span>
                </ShapeButton>
                <ShapeButton
                  variant="ghost"
                  onClick={() => handleShapeSelect("line")}
                >
                  <Minus className="h-5 w-5" />
                  <span>线条</span>
                </ShapeButton>
              </ShapeGrid>
            </PopoverContent>
          </Popover>

          <Tooltip>
            <TooltipTrigger asChild>
              <ToolButton
                variant="ghost"
                size="icon"
                onClick={() => handleToolClick("background")}
              >
                <Palette className="h-5 w-5" />
              </ToolButton>
            </TooltipTrigger>
            <TooltipContent>设置背景</TooltipContent>
          </Tooltip>

          {/* 对齐按钮 */}
          {onAlign && onToggleGridSnap && (
            <Popover open={alignPopoverOpen} onOpenChange={setAlignPopoverOpen}>
              <Tooltip>
                <TooltipTrigger asChild>
                  <PopoverTrigger asChild>
                    <ToolButton
                      variant="ghost"
                      size="icon"
                      $active={alignPopoverOpen}
                    >
                      <AlignCenterHorizontal className="h-5 w-5" />
                    </ToolButton>
                  </PopoverTrigger>
                </TooltipTrigger>
                <TooltipContent>对齐工具</TooltipContent>
              </Tooltip>
              <PopoverContent className="w-auto p-0" side="top">
                <AlignmentToolbar
                  hasSelection={hasSelection}
                  gridSnapEnabled={gridSnapEnabled}
                  onAlign={(direction) => {
                    onAlign(direction);
                    setAlignPopoverOpen(false);
                  }}
                  onToggleGridSnap={onToggleGridSnap}
                />
              </PopoverContent>
            </Popover>
          )}
        </ToolbarContainer>
      </TooltipProvider>
    );
  },
);

ElementToolbar.displayName = "ElementToolbar";
