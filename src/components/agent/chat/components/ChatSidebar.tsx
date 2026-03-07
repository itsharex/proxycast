import React, { useState } from "react";
import { MessageSquare, Plus, Trash2 } from "lucide-react";
import styled from "styled-components";
import type { Topic } from "../hooks/useAgentChat";

const SidebarContainer = styled.div`
  display: flex;
  flex-direction: column;
  height: 100%;
  margin-right: 10px;
  background-color: hsl(var(--muted) / 0.3);
  border-right: 1px solid hsl(var(--border));
`;

const Toolbar = styled.div`
  padding: 12px;
  border-bottom: 1px solid hsl(var(--border));
`;

const NewTopicButton = styled.button`
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: hsl(var(--foreground));
  background-color: transparent;
  border: 1px dashed hsl(var(--border));
  transition: all 0.2s;

  &:hover {
    background-color: hsl(var(--muted));
    border-color: hsl(var(--muted-foreground));
  }
`;

const ListContainer = styled.div`
  flex: 1;
  overflow-y: auto;
  padding: 8px;
`;

const ListItem = styled.div<{ $active: boolean }>`
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  margin-bottom: 4px;
  border-radius: 8px;
  cursor: pointer;
  background-color: ${(props) =>
    props.$active ? "hsl(var(--muted))" : "transparent"};
  color: ${(props) =>
    props.$active ? "hsl(var(--foreground))" : "hsl(var(--muted-foreground))"};
  transition: all 0.15s;

  &:hover {
    background-color: hsl(var(--muted));
    color: hsl(var(--foreground));
  }

  .title {
    font-size: 13px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    flex: 1;
  }

  .delete-btn {
    opacity: 0;
    transition: opacity 0.15s;
    padding: 4px;
    border-radius: 4px;
  }

  .delete-btn:hover {
    background-color: hsl(var(--destructive) / 0.15);
    color: hsl(var(--destructive));
  }

  &:hover .delete-btn {
    opacity: 1;
  }
`;

const EmptyState = styled.div`
  text-align: center;
  padding: 24px 16px;
  color: hsl(var(--muted-foreground));
  font-size: 13px;
`;

interface ChatSidebarProps {
  onNewChat: () => void;
  topics: Topic[];
  currentTopicId: string | null;
  onSwitchTopic: (topicId: string) => void;
  onDeleteTopic: (topicId: string) => void;
  onRenameTopic?: (topicId: string, newTitle: string) => void;
}

export const ChatSidebar: React.FC<ChatSidebarProps> = ({
  onNewChat,
  topics,
  currentTopicId,
  onSwitchTopic,
  onDeleteTopic,
  onRenameTopic,
}) => {
  const [editingTopicId, setEditingTopicId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState("");
  const editInputRef = React.useRef<HTMLInputElement>(null);

  const handleDeleteClick = (event: React.MouseEvent, topicId: string) => {
    event.stopPropagation();
    onDeleteTopic(topicId);
  };

  const handleStartEdit = (
    event: React.MouseEvent,
    topicId: string,
    currentTitle: string,
  ) => {
    event.stopPropagation();
    setEditingTopicId(topicId);
    setEditTitle(currentTitle);
  };

  const handleSaveEdit = () => {
    if (editingTopicId && editTitle.trim() && onRenameTopic) {
      onRenameTopic(editingTopicId, editTitle.trim());
    }
    setEditingTopicId(null);
    setEditTitle("");
  };

  const handleCancelEdit = () => {
    setEditingTopicId(null);
    setEditTitle("");
  };

  const handleEditKeyDown = (event: React.KeyboardEvent) => {
    if (event.key === "Enter") {
      handleSaveEdit();
    } else if (event.key === "Escape") {
      handleCancelEdit();
    }
  };

  React.useEffect(() => {
    if (editingTopicId && editInputRef.current) {
      editInputRef.current.focus();
      editInputRef.current.select();
    }
  }, [editingTopicId]);

  return (
    <SidebarContainer className="w-64 shrink-0">
      <Toolbar>
        <NewTopicButton onClick={onNewChat}>
          <Plus size={16} />
          <span>新建话题</span>
        </NewTopicButton>
      </Toolbar>

      <ListContainer className="custom-scrollbar">
        {topics.length === 0 ? (
          <EmptyState>暂无话题，点击上方新建</EmptyState>
        ) : (
          topics.map((topic) => (
            <ListItem
              key={topic.id}
              $active={topic.id === currentTopicId}
              onClick={() => {
                if (editingTopicId !== topic.id) {
                  onSwitchTopic(topic.id);
                }
              }}
              onDoubleClick={(event) =>
                handleStartEdit(event, topic.id, topic.title)
              }
            >
              <MessageSquare
                size={15}
                className={
                  topic.id === currentTopicId ? "text-primary" : "opacity-50"
                }
              />
              {editingTopicId === topic.id ? (
                <input
                  ref={editInputRef}
                  type="text"
                  value={editTitle}
                  onChange={(event) => setEditTitle(event.target.value)}
                  onKeyDown={handleEditKeyDown}
                  onBlur={handleSaveEdit}
                  onClick={(event) => event.stopPropagation()}
                  style={{
                    flex: 1,
                    fontSize: "13px",
                    padding: "2px 6px",
                    border: "1px solid hsl(var(--primary))",
                    borderRadius: "4px",
                    outline: "none",
                  }}
                />
              ) : (
                <span className="title">{topic.title}</span>
              )}
              {editingTopicId !== topic.id ? (
                <button
                  className="delete-btn"
                  onClick={(event) => handleDeleteClick(event, topic.id)}
                >
                  <Trash2 size={14} />
                </button>
              ) : null}
            </ListItem>
          ))
        )}
      </ListContainer>
    </SidebarContainer>
  );
};

