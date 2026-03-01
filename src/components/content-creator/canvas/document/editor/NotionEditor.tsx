import React, {
  memo,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from "react";
import { useEditor, EditorContent } from "@tiptap/react";
import { createExtensions } from "./extensions";
import { BubbleToolbar } from "./BubbleToolbar";
import {
  CommandList,
  type SlashMenuState,
  type SlashMenuKeyHandler,
} from "./SlashCommand";
import { markdownToHtml, htmlToMarkdown } from "./utils/markdown";
import "./editor-styles.css";

interface NotionEditorProps {
  content: string;
  onChange: (content: string) => void;
  onSave: () => void;
  onCancel: () => void;
  onSelectionTextChange?: (text: string) => void;
  externalImageInsert?: {
    requestId: string;
    url: string;
    alt?: string;
  } | null;
  onExternalImageInsertComplete?: (requestId: string, success: boolean) => void;
}

const EMPTY_SLASH: SlashMenuState = {
  isOpen: false,
  items: [],
  range: null,
  clientRect: null,
};

export const NotionEditor: React.FC<NotionEditorProps> = memo(
  ({
    content,
    onChange,
    onSave,
    onCancel,
    onSelectionTextChange,
    externalImageInsert,
    onExternalImageInsertComplete,
  }) => {
    const [slashState, setSlashState] = useState<SlashMenuState>(EMPTY_SLASH);
    const keyDownRef = useRef<SlashMenuKeyHandler | null>(null);
    const handledExternalInsertRef = useRef<string | null>(null);

    const extensions = useMemo(
      () =>
        createExtensions({
          onStateChange: setSlashState,
          onKeyDownRef: keyDownRef,
        }),
      [],
    );

    // eslint-disable-next-line react-hooks/exhaustive-deps
    const initialContent = useMemo(() => markdownToHtml(content), []);

    const editor = useEditor({
      extensions,
      content: initialContent,
      onUpdate: ({ editor }) => {
        onChange(htmlToMarkdown(editor.getHTML()));
      },
    });

    const handleSlashClose = useCallback(() => {
      setSlashState(EMPTY_SLASH);
    }, []);

    // 快捷键: Cmd+S 保存, Escape 取消
    useEffect(() => {
      const handleKeyDown = (e: KeyboardEvent) => {
        if ((e.metaKey || e.ctrlKey) && e.key === "s") {
          e.preventDefault();
          onSave();
        }
        if (e.key === "Escape" && !slashState.isOpen) {
          e.preventDefault();
          onCancel();
        }
      };

      document.addEventListener("keydown", handleKeyDown);
      return () => document.removeEventListener("keydown", handleKeyDown);
    }, [onSave, onCancel, slashState.isOpen]);

    useEffect(() => {
      if (editor) {
        editor.commands.focus("end");
      }
    }, [editor]);

    useEffect(() => {
      if (!editor || !onSelectionTextChange) {
        return;
      }

      const handleSelectionUpdate = () => {
        const { from, to, empty } = editor.state.selection;
        if (empty) {
          onSelectionTextChange("");
          return;
        }

        const selectedText = editor.state.doc.textBetween(from, to, "\n").trim();
        onSelectionTextChange(selectedText);
      };

      const handleBlur = () => {
        onSelectionTextChange("");
      };

      editor.on("selectionUpdate", handleSelectionUpdate);
      editor.on("blur", handleBlur);

      return () => {
        editor.off("selectionUpdate", handleSelectionUpdate);
        editor.off("blur", handleBlur);
      };
    }, [editor, onSelectionTextChange]);

    useEffect(() => {
      if (!editor || !externalImageInsert) {
        return;
      }

      if (handledExternalInsertRef.current === externalImageInsert.requestId) {
        return;
      }

      handledExternalInsertRef.current = externalImageInsert.requestId;
      const success = editor
        .chain()
        .focus()
        .setImage({
          src: externalImageInsert.url,
          alt: externalImageInsert.alt || "插图",
        })
        .run();

      if (success) {
        onChange(htmlToMarkdown(editor.getHTML()));
      }
      onExternalImageInsertComplete?.(externalImageInsert.requestId, success);
    }, [editor, externalImageInsert, onChange, onExternalImageInsertComplete]);

    if (!editor) return null;

    return (
      <div className="notion-editor-wrapper flex-1">
        <BubbleToolbar editor={editor} />
        <EditorContent editor={editor} />
        {slashState.isOpen && slashState.range && (
          <CommandList
            editor={editor}
            items={slashState.items}
            range={slashState.range}
            clientRect={slashState.clientRect}
            onKeyDownRef={keyDownRef}
            onClose={handleSlashClose}
          />
        )}
      </div>
    );
  },
);

NotionEditor.displayName = "NotionEditor";
