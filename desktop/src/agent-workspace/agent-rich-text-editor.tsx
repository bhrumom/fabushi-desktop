import { Node as TiptapNode, mergeAttributes, type Editor, type NodeViewRenderer } from '@tiptap/core';
import { Placeholder } from '@tiptap/extension-placeholder';
import type { Node as ProseMirrorNode } from '@tiptap/pm/model';
import { EditorContent, useEditor } from '@tiptap/react';
import { StarterKit } from '@tiptap/starter-kit';
import { useEffect, useMemo, useRef } from 'react';

export interface AgentRichTextDraft {
  readonly prompt: string;
  readonly richText?: string;
}

export interface AgentRichTextEditorControls {
  focus(): void;
  blur(): void;
  insertText(value: string): void;
}

export function agentEditorContent(prompt: string, richText?: string): Record<string, unknown> {
  if (richText) {
    try {
      const parsed: unknown = JSON.parse(richText);
      if (typeof parsed === 'object' && parsed !== null && (parsed as { type?: unknown }).type === 'doc') {
        return parsed as Record<string, unknown>;
      }
    } catch {
      // Invalid persisted JSON falls back to the plain prompt.
    }
  }
  return {
    type: 'doc',
    content: prompt.split('\n').map((line) => ({
      type: 'paragraph',
      ...(line ? { content: [{ type: 'text', text: line }] } : {}),
    })),
  };
}

function serializeEditorNode(node: ProseMirrorNode): string {
  if (node.isText) return node.text ?? '';
  if (node.type.name === 'hardBreak') return '\n';
  const toText = node.type.spec.toText;
  if (typeof toText === 'function') return toText({ node });
  let text = '';
  node.forEach((child, _offset, index) => {
    if (node.type.name === 'doc' && index > 0) text += '\n';
    text += serializeEditorNode(child);
  });
  return text;
}

export function agentEditorText(editor: Editor): string {
  return serializeEditorNode(editor.state.doc);
}

function simpleNodeView(className: string, marker: string): NodeViewRenderer {
  return ({ node }) => {
    const dom = document.createElement('span');
    dom.className = className;
    dom.dataset.type = className;
    dom.textContent = `${marker}${String(node.attrs.label ?? node.attrs.id ?? '')}`;
    return { dom };
  };
}

const AgentMention = TiptapNode.create({
  name: 'mention',
  group: 'inline',
  inline: true,
  atom: true,
  selectable: true,
  addAttributes() {
    return {
      id: { default: null },
      label: { default: null },
      kind: { default: 'agent' },
    };
  },
  parseHTML() {
    return [{ tag: 'span[data-type="agent-mention"]' }];
  },
  renderHTML({ node, HTMLAttributes }) {
    return ['span', mergeAttributes(HTMLAttributes, {
      'data-type': 'agent-mention',
      'data-id': node.attrs.id,
      'data-kind': node.attrs.kind,
      class: 'agent-mention',
    }), `@${node.attrs.label ?? node.attrs.id ?? ''}`];
  },
  renderText({ node }) {
    return `@${node.attrs.label ?? node.attrs.id ?? ''}`;
  },
  addNodeView() {
    return simpleNodeView('agent-mention', '@');
  },
});

const WorkflowReference = TiptapNode.create({
  name: 'workflowReference',
  group: 'inline',
  inline: true,
  atom: true,
  selectable: true,
  addAttributes() {
    return {
      id: { default: null },
      label: { default: null },
    };
  },
  parseHTML() {
    return [{ tag: 'span[data-type="agent-workflow-reference"]' }];
  },
  renderHTML({ node, HTMLAttributes }) {
    return ['span', mergeAttributes(HTMLAttributes, {
      'data-type': 'agent-workflow-reference',
      'data-id': node.attrs.id,
      class: 'agent-workflow-reference',
    }), `/${node.attrs.label ?? node.attrs.id ?? ''}`];
  },
  renderText({ node }) {
    return `/${node.attrs.label ?? node.attrs.id ?? ''}`;
  },
  addNodeView() {
    return simpleNodeView('agent-workflow-reference', '/');
  },
});

function filesFromClipboard(event: globalThis.ClipboardEvent): File[] {
  return Array.from(event.clipboardData?.items ?? [])
    .filter((item) => item.kind === 'file')
    .map((item) => item.getAsFile())
    .filter((file): file is File => file != null);
}

export default function AgentRichTextEditor({
  prompt,
  richText,
  scopeKey,
  disabled,
  placeholder,
  className,
  ariaLabel,
  onChange,
  onKeyDown,
  onPasteFiles,
  onControls,
}: {
  prompt: string;
  richText?: string;
  scopeKey: string;
  disabled: boolean;
  placeholder: string;
  className: string;
  ariaLabel: string;
  onChange(value: AgentRichTextDraft): void;
  onKeyDown?(event: globalThis.KeyboardEvent): boolean;
  onPasteFiles?(files: readonly File[]): void;
  onControls?(controls: AgentRichTextEditorControls | null): void;
}) {
  const callbacks = useRef({ onChange, onKeyDown, onPasteFiles });
  callbacks.current = { onChange, onKeyDown, onPasteFiles };
  const extensions = useMemo(() => [
    StarterKit.configure({
      blockquote: false,
      bold: false,
      bulletList: false,
      code: false,
      codeBlock: false,
      heading: false,
      horizontalRule: false,
      italic: false,
      link: false,
      orderedList: false,
      strike: false,
      undoRedo: { newGroupDelay: 100 },
    }),
    Placeholder.configure({
      placeholder,
      emptyEditorClass: 'is-agent-editor-empty',
      showOnlyWhenEditable: false,
    }),
    AgentMention,
    WorkflowReference,
  ], [placeholder]);

  const editor = useEditor({
    extensions,
    content: agentEditorContent(prompt, richText),
    autofocus: false,
    immediatelyRender: false,
    editable: !disabled,
    editorProps: {
      attributes: {
        class: className,
        'data-testid': 'messenger-input',
        'aria-label': ariaLabel,
        'aria-multiline': 'true',
        role: 'textbox',
        spellcheck: 'false',
      },
      handleKeyDown: (_view, event) => callbacks.current.onKeyDown?.(event) ?? false,
      handlePaste: (_view, event) => {
        const files = filesFromClipboard(event);
        if (!files.length || !callbacks.current.onPasteFiles) return false;
        event.preventDefault();
        callbacks.current.onPasteFiles(files);
        return true;
      },
    },
    onUpdate: ({ editor: current }) => {
      const promptValue = agentEditorText(current);
      const json = current.isEmpty ? undefined : JSON.stringify(current.getJSON());
      callbacks.current.onChange({ prompt: promptValue, ...(json ? { richText: json } : {}) });
    },
  }, [scopeKey, extensions]);

  useEffect(() => {
    if (!editor) return;
    editor.setEditable(!disabled);
  }, [disabled, editor]);

  useEffect(() => {
    if (!editor) return;
    const expected = agentEditorContent(prompt, richText);
    if (JSON.stringify(editor.getJSON()) === JSON.stringify(expected)) return;
    const wasFocused = editor.isFocused;
    editor.commands.setContent(expected, { emitUpdate: false });
    if (wasFocused) editor.commands.focus('end');
  }, [editor, prompt, richText]);

  useEffect(() => {
    if (!editor) return;
    const controls: AgentRichTextEditorControls = {
      focus: () => editor.commands.focus('end'),
      blur: () => editor.view.dom.blur(),
      insertText: (value) => {
        if (!value) return;
        editor.chain().focus().insertContent(value).run();
      },
    };
    onControls?.(controls);
    return () => onControls?.(null);
  }, [editor, onControls]);

  return <EditorContent editor={editor} />;
}
