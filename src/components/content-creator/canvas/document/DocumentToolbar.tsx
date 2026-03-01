/**
 * @file 文档工具栏组件
 * @description 提供版本选择、编辑、导出、关闭等功能
 * @module components/content-creator/canvas/document/DocumentToolbar
 */

import React, { memo, useState, useRef, useEffect } from "react";
import styled from "styled-components";
import type { DocumentToolbarProps, ExportFormat } from "./types";
import { VersionSelector } from "./VersionSelector";

const Container = styled.div`
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: hsl(var(--background));
  border-bottom: 1px solid hsl(var(--border));
`;

const LeftSection = styled.div`
  display: flex;
  align-items: center;
  gap: 12px;
`;

const RightSection = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
`;

const ToolButton = styled.button`
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid hsl(var(--border));
  border-radius: 6px;
  background: hsl(var(--background));
  color: hsl(var(--foreground));
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: hsl(var(--muted) / 0.5);
  }
`;

const PrimaryButton = styled(ToolButton)`
  background: hsl(var(--primary));
  color: hsl(var(--primary-foreground));
  border-color: hsl(var(--primary));

  &:hover {
    background: hsl(var(--primary) / 0.9);
  }
`;

const CloseButton = styled.button`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: hsl(var(--muted-foreground));
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s;

  &:hover {
    background: hsl(var(--muted) / 0.5);
    color: hsl(var(--foreground));
  }
`;

const ExportDropdown = styled.div`
  position: relative;
`;

const DropdownMenu = styled.div<{ $visible: boolean }>`
  position: absolute;
  top: 100%;
  right: 0;
  margin-top: 4px;
  min-width: 140px;
  background: hsl(var(--background));
  border: 1px solid hsl(var(--border));
  border-radius: 8px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 100;
  opacity: ${({ $visible }) => ($visible ? 1 : 0)};
  visibility: ${({ $visible }) => ($visible ? "visible" : "hidden")};
  transform: ${({ $visible }) =>
    $visible ? "translateY(0)" : "translateY(-8px)"};
  transition: all 0.2s;
`;

const DropdownItem = styled.button`
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  background: transparent;
  color: hsl(var(--foreground));
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;

  &:hover {
    background: hsl(var(--muted) / 0.3);
  }

  &:first-child {
    border-radius: 8px 8px 0 0;
  }

  &:last-child {
    border-radius: 0 0 8px 8px;
  }
`;

const Title = styled.h3`
  font-size: 14px;
  font-weight: 600;
  color: hsl(var(--foreground));
  margin: 0;
`;

/**
 * 文档工具栏组件
 */
export const DocumentToolbar: React.FC<DocumentToolbarProps> = memo(
  ({
    currentVersion,
    versions,
    isEditing,
    onVersionChange,
    onEditToggle,
    onSave,
    onCancel,
    onExport,
    onAutoInsertImages,
    autoInsertLoading = false,
    onClose,
  }) => {
    const [showExportMenu, setShowExportMenu] = useState(false);
    const exportRef = useRef<HTMLDivElement>(null);

    // 点击外部关闭导出菜单
    useEffect(() => {
      const handleClickOutside = (event: MouseEvent) => {
        if (
          exportRef.current &&
          !exportRef.current.contains(event.target as Node)
        ) {
          setShowExportMenu(false);
        }
      };
      document.addEventListener("mousedown", handleClickOutside);
      return () =>
        document.removeEventListener("mousedown", handleClickOutside);
    }, []);

    const handleExport = (format: ExportFormat) => {
      onExport(format);
      setShowExportMenu(false);
    };

    return (
      <Container>
        <LeftSection>
          <Title>文档</Title>
          <VersionSelector
            currentVersion={currentVersion}
            versions={versions}
            onVersionChange={onVersionChange}
          />
        </LeftSection>

        <RightSection>
          {isEditing ? (
            <>
              <ToolButton onClick={onCancel}>取消</ToolButton>
              <PrimaryButton onClick={onSave}>💾 保存</PrimaryButton>
            </>
          ) : (
            <>
              <ToolButton onClick={onEditToggle}>✏️ 编辑</ToolButton>
              {onAutoInsertImages && (
                <ToolButton
                  onClick={onAutoInsertImages}
                  disabled={autoInsertLoading}
                  title="按主题自动搜索并插入图片"
                >
                  {autoInsertLoading ? "🖼️ 配图中..." : "🖼️ 主题配图"}
                </ToolButton>
              )}
              <ExportDropdown ref={exportRef}>
                <ToolButton onClick={() => setShowExportMenu(!showExportMenu)}>
                  📤 导出 ▼
                </ToolButton>
                <DropdownMenu $visible={showExportMenu}>
                  <DropdownItem onClick={() => handleExport("markdown")}>
                    📄 Markdown (.md)
                  </DropdownItem>
                  <DropdownItem onClick={() => handleExport("text")}>
                    📝 纯文本 (.txt)
                  </DropdownItem>
                  <DropdownItem onClick={() => handleExport("clipboard")}>
                    📋 复制到剪贴板
                  </DropdownItem>
                </DropdownMenu>
              </ExportDropdown>
            </>
          )}
          <CloseButton onClick={onClose} title="关闭">
            ✕
          </CloseButton>
        </RightSection>
      </Container>
    );
  },
);

DocumentToolbar.displayName = "DocumentToolbar";
