/**
 * @file CommandApproval.tsx
 * @description 命令审批组件 - AI 执行命令前的用户确认
 * @module components/terminal/ai/CommandApproval
 *
 * 当 AI 需要在终端执行命令时，显示审批对话框让用户确认。
 * 参考 Waveterm 的工具调用审批流程。
 */

import React from "react";
import styled from "styled-components";
import type { PendingCommand } from "./TerminalController";

// ============================================================================
// 样式组件
// ============================================================================

const ApprovalContainer = styled.div`
  background: #1e293b;
  border: 1px solid #334155;
  border-radius: 8px;
  padding: 12px;
  margin: 8px 0;
`;

const ApprovalHeader = styled.div`
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
  color: #f59e0b;
  font-size: 13px;
  font-weight: 500;
`;

const WarningIcon = () => (
  <svg
    width="16"
    height="16"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    strokeWidth="2"
  >
    <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
    <line x1="12" y1="9" x2="12" y2="13" />
    <line x1="12" y1="17" x2="12.01" y2="17" />
  </svg>
);

const CommandBox = styled.div`
  background: #0f172a;
  border: 1px solid #1e293b;
  border-radius: 6px;
  padding: 10px 12px;
  font-family: "Hack", "Menlo", monospace;
  font-size: 13px;
  color: #e2e8f0;
  margin-bottom: 12px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-break: break-all;
`;

const ButtonGroup = styled.div`
  display: flex;
  gap: 8px;
  justify-content: flex-end;
`;

const Button = styled.button<{ $variant?: "approve" | "reject" }>`
  padding: 6px 16px;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.15s ease;
  border: none;

  ${({ $variant }) =>
    $variant === "approve"
      ? `
    background: #22c55e;
    color: white;
    &:hover {
      background: #16a34a;
    }
  `
      : `
    background: #334155;
    color: #94a3b8;
    &:hover {
      background: #475569;
      color: #e2e8f0;
    }
  `}
`;

const HelpText = styled.p`
  font-size: 12px;
  color: #64748b;
  margin: 0 0 12px 0;
`;

// ============================================================================
// 组件
// ============================================================================

interface CommandApprovalProps {
  /** 待审批的命令 */
  command: PendingCommand;
  /** 批准回调 */
  onApprove: (commandId: string) => void;
  /** 拒绝回调 */
  onReject: (commandId: string) => void;
}

/**
 * 命令审批组件
 */
export function CommandApproval({
  command,
  onApprove,
  onReject,
}: CommandApprovalProps) {
  return (
    <ApprovalContainer>
      <ApprovalHeader>
        <WarningIcon />
        AI 请求执行命令
      </ApprovalHeader>

      <CommandBox>{command.command}</CommandBox>

      <HelpText>此命令将在当前终端中执行。请确认是否允许。</HelpText>

      <ButtonGroup>
        <Button onClick={() => onReject(command.id)}>拒绝</Button>
        <Button $variant="approve" onClick={() => onApprove(command.id)}>
          执行
        </Button>
      </ButtonGroup>
    </ApprovalContainer>
  );
}

// ============================================================================
// 命令列表组件
// ============================================================================

interface CommandApprovalListProps {
  /** 待审批的命令列表 */
  commands: PendingCommand[];
  /** 批准回调 */
  onApprove: (commandId: string) => void;
  /** 拒绝回调 */
  onReject: (commandId: string) => void;
}

/**
 * 命令审批列表组件
 */
export function CommandApprovalList({
  commands,
  onApprove,
  onReject,
}: CommandApprovalListProps) {
  if (commands.length === 0) {
    return null;
  }

  return (
    <>
      {commands.map((cmd) => (
        <CommandApproval
          key={cmd.id}
          command={cmd}
          onApprove={onApprove}
          onReject={onReject}
        />
      ))}
    </>
  );
}

export default CommandApproval;
