"use client";

import {
  AppWindow,
  Check,
  ChevronDown,
  ChevronUp,
  Copy,
  ExternalLink,
  FileSpreadsheet,
  FileText,
  GitBranch,
  KeyRound,
  Mail,
  Maximize2,
  MessageSquare,
  Minimize2,
  RotateCcw,
  Send,
  Trash2,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import React, {
  Fragment,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
  type ReactNode,
} from "react";
import type {
  EmailDraft,
  ListenerPlatform,
  MessageDraft,
  SlackDraft,
  TranscriptCard,
} from "../../lib/mahayana-host/contracts";
import styles from "./host.module.css";

export interface TranscriptCardEntry {
  entryId: string;
  operationId?: string;
  card: TranscriptCard;
}

interface TranscriptCardViewProps {
  entry: TranscriptCardEntry;
  onResolveDraft: (draft: MessageDraft, action: "send" | "discard") => void;
  onProvideSecret: (requestId: string, value: string) => void;
  onConnectListener: (platform: ListenerPlatform) => void;
}

const listenerCopy: Record<ListenerPlatform, [string, string]> = {
  github: ["GitHub", "Let automations watch a repo's PRs, comments, issues, and CI."],
  git: ["Git", "Wake automations on local commits, branches, tags, and repository changes."],
  slack: ["Slack", "Wake automations on Slack messages, mentions, and reactions."],
  teams: ["Microsoft Teams", "Wake automations on Teams messages, mentions, and reactions."],
  linear: ["Linear", "Wake automations on issues, comments, status changes, and assignments."],
  sentry: ["Sentry", "Wake automations on new, regressed, assigned, and resolved issues."],
  pagerduty: ["PagerDuty", "Wake automations when incidents are triggered, acknowledged, escalated, or resolved."],
};

const safeHref = (value: string): string | null => {
  if (value.startsWith("/") || value.startsWith("#")) return value;
  try {
    const url = new URL(value);
    return ["http:", "https:", "mailto:"].includes(url.protocol) ? url.href : null;
  } catch {
    return null;
  }
};

const safeDocumentSource = (value: string | undefined): string | undefined => {
  if (!value) return undefined;
  if (value.startsWith("data:application/pdf;base64,")) return value;
  try {
    const url = new URL(value);
    return ["http:", "https:", "blob:", "tauri:"].includes(url.protocol)
      ? url.href
      : undefined;
  } catch {
    return undefined;
  }
};

const inlinePattern = /(\[[^\]]+\]\([^)]+\)|`[^`]+`|\*\*[^*]+\*\*|__[^_]+__|~~[^~]+~~|\*[^*]+\*)/g;

function renderInline(text: string): ReactNode[] {
  const parts = text.split(inlinePattern).filter(Boolean);
  return parts.map((part, index) => {
    if (part.startsWith("[") && part.includes("](")) {
      const match = part.match(/^\[([^\]]+)\]\(([^)]+)\)$/);
      const href = match ? safeHref(match[2]) : null;
      return href ? (
        <a key={`${part}-${index}`} href={href} target="_blank" rel="noreferrer">
          {match?.[1]}
        </a>
      ) : (
        <Fragment key={`${part}-${index}`}>{match?.[1] ?? part}</Fragment>
      );
    }
    if (part.startsWith("`") && part.endsWith("`")) {
      return <code key={`${part}-${index}`}>{part.slice(1, -1)}</code>;
    }
    if (
      (part.startsWith("**") && part.endsWith("**")) ||
      (part.startsWith("__") && part.endsWith("__"))
    ) {
      return <strong key={`${part}-${index}`}>{part.slice(2, -2)}</strong>;
    }
    if (part.startsWith("~~") && part.endsWith("~~")) {
      return <del key={`${part}-${index}`}>{part.slice(2, -2)}</del>;
    }
    if (part.startsWith("*") && part.endsWith("*")) {
      return <em key={`${part}-${index}`}>{part.slice(1, -1)}</em>;
    }
    return <Fragment key={`${part}-${index}`}>{part}</Fragment>;
  });
}

interface MarkdownSegment {
  kind: "text" | "code";
  value: string;
  language?: string;
}

function splitMarkdown(text: string): MarkdownSegment[] {
  const segments: MarkdownSegment[] = [];
  const pattern = /```([^\n`]*)\n([\s\S]*?)```/g;
  let cursor = 0;
  for (const match of text.matchAll(pattern)) {
    const index = match.index ?? 0;
    if (index > cursor) segments.push({ kind: "text", value: text.slice(cursor, index) });
    segments.push({
      kind: "code",
      language: match[1].trim().toLowerCase() || "text",
      value: match[2].replace(/\n$/, ""),
    });
    cursor = index + match[0].length;
  }
  if (cursor < text.length) segments.push({ kind: "text", value: text.slice(cursor) });
  return segments;
}

function isTableDelimiter(line: string): boolean {
  const cells = line.trim().replace(/^\||\|$/g, "").split("|");
  return cells.length > 1 && cells.every((cell) => /^\s*:?-{3,}:?\s*$/.test(cell));
}

function tableCells(line: string): string[] {
  return line.trim().replace(/^\||\|$/g, "").split("|").map((cell) => cell.trim());
}

function MarkdownText({ text }: { text: string }) {
  const blocks: ReactNode[] = [];
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  let index = 0;
  while (index < lines.length) {
    const line = lines[index];
    if (!line.trim()) {
      index += 1;
      continue;
    }
    if (index + 1 < lines.length && line.includes("|") && isTableDelimiter(lines[index + 1])) {
      const header = tableCells(line);
      const rows: string[][] = [];
      index += 2;
      while (index < lines.length && lines[index].includes("|") && lines[index].trim()) {
        rows.push(tableCells(lines[index]));
        index += 1;
      }
      blocks.push(
        <div className={styles.richTableWrap} key={`table-${index}`}>
          <table className={styles.richTable}>
            <thead><tr>{header.map((cell, cellIndex) => <th key={cellIndex}>{renderInline(cell)}</th>)}</tr></thead>
            <tbody>{rows.map((row, rowIndex) => <tr key={rowIndex}>{header.map((_, cellIndex) => <td key={cellIndex}>{renderInline(row[cellIndex] ?? "")}</td>)}</tr>)}</tbody>
          </table>
        </div>,
      );
      continue;
    }
    const heading = line.match(/^(#{1,6})\s+(.+)$/);
    if (heading) {
      const level = heading[1].length;
      blocks.push(
        <div className={styles[`richHeading${Math.min(level, 3)}` as keyof typeof styles]} key={`heading-${index}`}>
          {renderInline(heading[2])}
        </div>,
      );
      index += 1;
      continue;
    }
    if (/^\s*[-*_]{3,}\s*$/.test(line)) {
      blocks.push(<hr className={styles.richRule} key={`rule-${index}`} />);
      index += 1;
      continue;
    }
    if (/^\s*>\s?/.test(line)) {
      const quote: string[] = [];
      while (index < lines.length && /^\s*>\s?/.test(lines[index])) {
        quote.push(lines[index].replace(/^\s*>\s?/, ""));
        index += 1;
      }
      blocks.push(<blockquote className={styles.richQuote} key={`quote-${index}`}>{quote.map((item, row) => <p key={row}>{renderInline(item)}</p>)}</blockquote>);
      continue;
    }
    if (/^\s*[-+*]\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*[-+*]\s+/.test(lines[index])) {
        items.push(lines[index].replace(/^\s*[-+*]\s+/, ""));
        index += 1;
      }
      const hasTasks = items.some((item) => /^\[[ xX]\]\s+/.test(item));
      blocks.push(
        <ul className={`${styles.richList} ${hasTasks ? styles.richTaskList : ""}`} key={`ul-${index}`}>
          {items.map((item, row) => {
            const task = item.match(/^\[([ xX])\]\s+(.+)$/);
            return (
              <li className={task ? styles.richTaskItem : undefined} key={row}>
                {task ? (
                  <>
                    <input type="checkbox" checked={task[1].toLowerCase() === "x"} readOnly />
                    <span>{renderInline(task[2])}</span>
                  </>
                ) : renderInline(item)}
              </li>
            );
          })}
        </ul>,
      );
      continue;
    }
    if (/^\s*\d+[.)]\s+/.test(line)) {
      const items: string[] = [];
      while (index < lines.length && /^\s*\d+[.)]\s+/.test(lines[index])) {
        items.push(lines[index].replace(/^\s*\d+[.)]\s+/, ""));
        index += 1;
      }
      blocks.push(<ol className={styles.richList} key={`ol-${index}`}>{items.map((item, row) => <li key={row}>{renderInline(item)}</li>)}</ol>);
      continue;
    }
    const paragraph: string[] = [line.trim()];
    index += 1;
    while (
      index < lines.length &&
      lines[index].trim() &&
      !/^(#{1,6})\s+/.test(lines[index]) &&
      !/^\s*(?:[-+*]|\d+[.)])\s+/.test(lines[index]) &&
      !/^\s*>\s?/.test(lines[index]) &&
      !(index + 1 < lines.length && lines[index].includes("|") && isTableDelimiter(lines[index + 1]))
    ) {
      paragraph.push(lines[index].trim());
      index += 1;
    }
    blocks.push(<p className={styles.richParagraph} key={`p-${index}`}>{renderInline(paragraph.join(" "))}</p>);
  }
  return <>{blocks}</>;
}

const languageKeywords: Record<string, Set<string>> = {
  js: new Set(["const", "let", "var", "function", "return", "if", "else", "for", "while", "async", "await", "import", "export", "class", "new", "throw", "try", "catch"]),
  javascript: new Set(["const", "let", "var", "function", "return", "if", "else", "for", "while", "async", "await", "import", "export", "class", "new", "throw", "try", "catch"]),
  ts: new Set(["const", "let", "interface", "type", "function", "return", "if", "else", "async", "await", "import", "export", "class", "extends", "implements", "unknown", "never"]),
  typescript: new Set(["const", "let", "interface", "type", "function", "return", "if", "else", "async", "await", "import", "export", "class", "extends", "implements", "unknown", "never"]),
  rust: new Set(["fn", "let", "mut", "pub", "impl", "struct", "enum", "match", "if", "else", "use", "mod", "trait", "where", "async", "await", "return", "Self", "self"]),
  py: new Set(["def", "class", "return", "if", "elif", "else", "for", "while", "import", "from", "as", "try", "except", "with", "async", "await", "yield", "None", "True", "False"]),
  python: new Set(["def", "class", "return", "if", "elif", "else", "for", "while", "import", "from", "as", "try", "except", "with", "async", "await", "yield", "None", "True", "False"]),
};

function highlightedLine(line: string, language: string): ReactNode[] {
  const keywords = languageKeywords[language] ?? new Set<string>();
  const parts = line.split(/(\/\/.*$|#.*$|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'|\b\d+(?:\.\d+)?\b|\b[A-Za-z_$][\w$]*\b)/g).filter(Boolean);
  return parts.map((part, index) => {
    const className =
      part.startsWith("//") || (part.startsWith("#") && language !== "css")
        ? styles.codeComment
        : /^['"]/.test(part)
          ? styles.codeString
          : /^\d/.test(part)
            ? styles.codeNumber
            : keywords.has(part)
              ? styles.codeKeyword
              : undefined;
    return <span className={className} key={`${index}-${part}`}>{part}</span>;
  });
}

function CodeBlock({ code, language }: { code: string; language: string }) {
  const [copied, setCopied] = useState(false);
  const lines = code.split("\n");
  return (
    <section className={styles.codeCard}>
      <header className={styles.codeHeader}>
        <span>{language || "text"}</span>
        <button type="button" onClick={() => void navigator.clipboard.writeText(code).then(() => { setCopied(true); window.setTimeout(() => setCopied(false), 1200); })}>
          {copied ? <Check size={14} /> : <Copy size={14} />} {copied ? "Copied" : "Copy"}
        </button>
      </header>
      <pre className={styles.codeBody}>{lines.map((line, index) => <div className={styles.codeLine} key={index}><span className={styles.codeLineNumber}>{index + 1}</span><code>{highlightedLine(line, language)}</code></div>)}</pre>
    </section>
  );
}

function DiffBlock({ diff }: { diff: string }) {
  return (
    <section className={styles.diffCard}>
      <header className={styles.codeHeader}><span>diff</span></header>
      <pre className={styles.diffBody}>{diff.split("\n").map((line, index) => {
        const kind = line.startsWith("+") && !line.startsWith("+++") ? "add" : line.startsWith("-") && !line.startsWith("---") ? "remove" : line.startsWith("@@") ? "hunk" : "context";
        return <div className={`${styles.diffLine} ${styles[`diff${kind[0].toUpperCase()}${kind.slice(1)}` as keyof typeof styles]}`} key={index}><span>{index + 1}</span><code>{line || " "}</code></div>;
      })}</pre>
    </section>
  );
}

interface MermaidNode { id: string; label: string; shape: "rect" | "round" | "diamond" }
interface MermaidEdge { from: string; to: string; label?: string }

function parseMermaidNode(token: string): MermaidNode {
  const id = token.match(/^[A-Za-z0-9_.-]+/)?.[0] ?? token.trim();
  const diamond = token.match(/\{([^}]+)\}/);
  const round = token.match(/\(([^)]+)\)/);
  const rect = token.match(/\[([^\]]+)\]/);
  return { id, label: diamond?.[1] ?? round?.[1] ?? rect?.[1] ?? id, shape: diamond ? "diamond" : round ? "round" : "rect" };
}

interface ParsedFlowDiagram {
  nodes: MermaidNode[];
  edges: MermaidEdge[];
  direction: "TB" | "BT" | "LR" | "RL";
}

interface SequenceParticipant {
  id: string;
  label: string;
}

interface ParsedSequenceDiagram {
  participants: SequenceParticipant[];
  messages: Array<{ from: string; to: string; label: string; dashed: boolean }>;
}

type ParsedMermaidDiagram =
  | { kind: "flow"; diagram: ParsedFlowDiagram }
  | { kind: "sequence"; diagram: ParsedSequenceDiagram }
  | { kind: "error"; message: string };

function parseFlowDiagram(source: string): ParsedFlowDiagram {
  const nodes = new Map<string, MermaidNode>();
  const edges: MermaidEdge[] = [];
  const header = source.trimStart().split("\n")[0]?.trim().split(/\s+/) ?? [];
  const requestedDirection = header[1]?.toUpperCase();
  const direction = ["TB", "BT", "LR", "RL"].includes(requestedDirection)
    ? requestedDirection as ParsedFlowDiagram["direction"]
    : "TB";
  for (const raw of source.split("\n").slice(1)) {
    const line = raw.trim();
    if (!line || line.startsWith("%%")) continue;
    const match = line.match(/^(.+?)\s*(?:-->|---|==>)\s*(?:\|([^|]+)\|\s*)?(.+)$/);
    if (!match) continue;
    const from = parseMermaidNode(match[1].trim());
    const to = parseMermaidNode(match[3].trim());
    nodes.set(from.id, from);
    nodes.set(to.id, to);
    edges.push({ from: from.id, to: to.id, label: match[2]?.trim() });
  }
  return { nodes: [...nodes.values()], edges, direction };
}

function parseSequenceDiagram(source: string): ParsedSequenceDiagram {
  const participants = new Map<string, SequenceParticipant>();
  const messages: ParsedSequenceDiagram["messages"] = [];
  const ensureParticipant = (id: string, label = id) => {
    if (!participants.has(id)) participants.set(id, { id, label });
  };
  for (const raw of source.split("\n").slice(1)) {
    const line = raw.trim();
    if (!line || line.startsWith("%%")) continue;
    const participant = line.match(/^participant\s+([^\s]+)(?:\s+as\s+(.+))?$/i);
    if (participant) {
      ensureParticipant(participant[1], participant[2]?.trim() || participant[1]);
      continue;
    }
    const message = line.match(/^([^\s]+)\s*(-{1,2}>>?)\s*([^:]+):\s*(.+)$/);
    if (!message) continue;
    const from = message[1].trim();
    const to = message[3].trim();
    ensureParticipant(from);
    ensureParticipant(to);
    messages.push({
      from,
      to,
      label: message[4].trim(),
      dashed: message[2].startsWith("--"),
    });
  }
  return { participants: [...participants.values()], messages };
}

function parseMermaidDiagram(source: string): ParsedMermaidDiagram {
  const first = source.trimStart().split(/\s+/)[0]?.toLowerCase();
  if (first === "flowchart" || first === "graph") {
    const diagram = parseFlowDiagram(source);
    return diagram.nodes.length
      ? { kind: "flow", diagram }
      : { kind: "error", message: "No supported flowchart edges were found." };
  }
  if (first === "sequencediagram") {
    const diagram = parseSequenceDiagram(source);
    return diagram.participants.length
      ? { kind: "sequence", diagram }
      : { kind: "error", message: "No sequence participants or messages were found." };
  }
  return {
    kind: "error",
    message: first
      ? `Unsupported Mermaid diagram type: ${first}.`
      : "The Mermaid source is empty.",
  };
}

function FlowDiagram({ diagram, markerId }: { diagram: ParsedFlowDiagram; markerId: string }) {
  const horizontal = diagram.direction === "LR" || diagram.direction === "RL";
  const width = horizontal ? Math.max(640, diagram.nodes.length * 240 + 40) : 640;
  const height = horizontal ? 210 : Math.max(160, diagram.nodes.length * 92 + 36);
  const orderedNodes = diagram.direction === "BT" || diagram.direction === "RL"
    ? [...diagram.nodes].reverse()
    : diagram.nodes;
  const positions = new Map(
    orderedNodes.map((node, index) => [
      node.id,
      horizontal
        ? { x: 140 + index * 240, y: height / 2 }
        : { x: width / 2, y: 54 + index * 92 },
    ]),
  );
  return (
    <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Mermaid flow diagram">
      <defs>
        <marker id={markerId} viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" />
        </marker>
      </defs>
      {diagram.edges.map((edge, index) => {
        const from = positions.get(edge.from);
        const to = positions.get(edge.to);
        if (!from || !to) return null;
        const line = horizontal
          ? { x1: from.x + 110, y1: from.y, x2: to.x - 110, y2: to.y }
          : { x1: from.x, y1: from.y + 28, x2: to.x, y2: to.y - 28 };
        return (
          <g key={`${edge.from}-${edge.to}-${index}`}>
            <line {...line} markerEnd={`url(#${markerId})`} />
            {edge.label ? (
              <text x={(from.x + to.x) / 2 + (horizontal ? 0 : 8)} y={(from.y + to.y) / 2 - (horizontal ? 9 : 0)} textAnchor={horizontal ? "middle" : undefined}>
                {edge.label}
              </text>
            ) : null}
          </g>
        );
      })}
      {diagram.nodes.map((node) => {
        const position = positions.get(node.id);
        if (!position) return null;
        return (
          <g key={node.id}>
            {node.shape === "diamond" ? (
              <polygon points={`${position.x},${position.y - 28} ${position.x + 100},${position.y} ${position.x},${position.y + 28} ${position.x - 100},${position.y}`} />
            ) : (
              <rect x={position.x - 110} y={position.y - 28} width="220" height="56" rx={node.shape === "round" ? 28 : 10} />
            )}
            <text x={position.x} y={position.y + 5} textAnchor="middle">{node.label}</text>
          </g>
        );
      })}
    </svg>
  );
}

function SequenceDiagram({ diagram, markerId }: { diagram: ParsedSequenceDiagram; markerId: string }) {
  const width = Math.max(640, diagram.participants.length * 190);
  const height = Math.max(180, 100 + diagram.messages.length * 72);
  const x = (id: string) => 95 + diagram.participants.findIndex((participant) => participant.id === id) * 190;
  return (
    <svg viewBox={`0 0 ${width} ${height}`} role="img" aria-label="Mermaid sequence diagram">
      <defs>
        <marker id={markerId} viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
          <path d="M 0 0 L 10 5 L 0 10 z" />
        </marker>
      </defs>
      {diagram.participants.map((participant) => (
        <g key={participant.id}>
          <rect x={x(participant.id) - 72} y="20" width="144" height="42" rx="9" />
          <text x={x(participant.id)} y="46" textAnchor="middle">{participant.label}</text>
          <line x1={x(participant.id)} y1="62" x2={x(participant.id)} y2={height - 20} className={styles.mermaidLifeLine} />
        </g>
      ))}
      {diagram.messages.map((message, index) => {
        const y = 96 + index * 72;
        return (
          <g key={`${message.from}-${message.to}-${index}`}>
            <line x1={x(message.from)} y1={y} x2={x(message.to)} y2={y} markerEnd={`url(#${markerId})`} strokeDasharray={message.dashed ? "7 5" : undefined} />
            <text x={(x(message.from) + x(message.to)) / 2} y={y - 9} textAnchor="middle">{message.label}</text>
          </g>
        );
      })}
    </svg>
  );
}

function MermaidBlock({ source }: { source: string }) {
  const parsed = useMemo(() => parseMermaidDiagram(source), [source]);
  const markerRoot = useId().replace(/:/g, "");
  const [copied, setCopied] = useState(false);
  const [expanded, setExpanded] = useState(false);
  const [showSource, setShowSource] = useState(false);
  const [zoom, setZoom] = useState(1);
  const [offset, setOffset] = useState({ x: 0, y: 0 });
  const [dragging, setDragging] = useState(false);
  const dragState = useRef<{
    pointerId: number;
    startX: number;
    startY: number;
    originX: number;
    originY: number;
  } | null>(null);

  useEffect(() => {
    if (!expanded) return;
    const closeOnEscape = (event: KeyboardEvent) => {
      if (event.key === "Escape") setExpanded(false);
    };
    window.addEventListener("keydown", closeOnEscape);
    return () => window.removeEventListener("keydown", closeOnEscape);
  }, [expanded]);

  const resetView = () => {
    setZoom(1);
    setOffset({ x: 0, y: 0 });
  };
  const adjustZoom = (delta: number) => {
    setZoom((current) => Math.min(2.5, Math.max(.5, Number((current + delta).toFixed(2)))));
  };
  const copySource = () => {
    void navigator.clipboard.writeText(source).then(() => {
      setCopied(true);
      window.setTimeout(() => setCopied(false), 1200);
    });
  };
  const startDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (parsed.kind === "error" || showSource) return;
    dragState.current = {
      pointerId: event.pointerId,
      startX: event.clientX,
      startY: event.clientY,
      originX: offset.x,
      originY: offset.y,
    };
    event.currentTarget.setPointerCapture(event.pointerId);
    setDragging(true);
  };
  const moveDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    const drag = dragState.current;
    if (!drag || drag.pointerId !== event.pointerId) return;
    setOffset({
      x: drag.originX + event.clientX - drag.startX,
      y: drag.originY + event.clientY - drag.startY,
    });
  };
  const stopDrag = (event: ReactPointerEvent<HTMLDivElement>) => {
    if (dragState.current?.pointerId !== event.pointerId) return;
    dragState.current = null;
    setDragging(false);
    if (event.currentTarget.hasPointerCapture(event.pointerId)) {
      event.currentTarget.releasePointerCapture(event.pointerId);
    }
  };

  const renderCard = (isExpanded: boolean) => {
    const markerId = `${markerRoot}-${isExpanded ? "expanded" : "inline"}`;
    return (
      <section className={`${styles.mermaidCard} ${isExpanded ? styles.mermaidExpanded : ""}`}>
        <header className={styles.mermaidHeader}>
          <span className={styles.mermaidTitle}>
            <GitBranch size={15} />
            Mermaid{parsed.kind === "sequence" ? " sequence" : ""}
          </span>
          <span className={styles.mermaidToolbar}>
            <button type="button" onClick={copySource} title="Copy Mermaid source">
              {copied ? <Check size={14} /> : <Copy size={14} />}<span>{copied ? "Copied" : "Copy"}</span>
            </button>
            <button type="button" onClick={() => setShowSource((current) => !current)} title={showSource ? "Show diagram" : "Show source"}>
              <FileText size={14} /><span>{showSource ? "Diagram" : "Source"}</span>
            </button>
            {parsed.kind !== "error" && !showSource ? (
              <>
                <button type="button" onClick={() => adjustZoom(-.15)} disabled={zoom <= .5} title="Zoom out"><ZoomOut size={14} /></button>
                <span className={styles.mermaidZoomLabel}>{Math.round(zoom * 100)}%</span>
                <button type="button" onClick={() => adjustZoom(.15)} disabled={zoom >= 2.5} title="Zoom in"><ZoomIn size={14} /></button>
                <button type="button" onClick={resetView} disabled={zoom === 1 && offset.x === 0 && offset.y === 0} title="Reset view"><RotateCcw size={14} /></button>
              </>
            ) : null}
            <button type="button" onClick={() => setExpanded(!isExpanded)} title={isExpanded ? "Exit expanded view" : "Expand diagram"}>
              {isExpanded ? <Minimize2 size={14} /> : <Maximize2 size={14} />}
            </button>
          </span>
        </header>
        {parsed.kind === "error" ? (
          <div className={styles.mermaidError} role="alert">
            <strong>Unable to render Mermaid diagram</strong>
            <span>{parsed.message}</span>
            <pre><code>{source}</code></pre>
          </div>
        ) : showSource ? (
          <pre className={styles.mermaidSource}><code>{source}</code></pre>
        ) : (
          <div
            className={`${styles.mermaidViewport} ${dragging ? styles.mermaidDragging : ""}`}
            onPointerDown={startDrag}
            onPointerMove={moveDrag}
            onPointerUp={stopDrag}
            onPointerCancel={stopDrag}
          >
            <div
              className={styles.mermaidCanvas}
              style={{ transform: `translate(${offset.x}px, ${offset.y}px) scale(${zoom})` }}
            >
              {parsed.kind === "sequence" ? (
                <SequenceDiagram diagram={parsed.diagram} markerId={markerId} />
              ) : (
                <FlowDiagram diagram={parsed.diagram} markerId={markerId} />
              )}
            </div>
          </div>
        )}
      </section>
    );
  };

  return (
    <>
      {renderCard(false)}
      {expanded ? (
        <div className={styles.mermaidBackdrop} role="dialog" aria-modal="true" aria-label="Expanded Mermaid diagram" onMouseDown={() => setExpanded(false)}>
          <div className={styles.mermaidModal} onMouseDown={(event) => event.stopPropagation()}>
            {renderCard(true)}
          </div>
        </div>
      ) : null}
    </>
  );
}

export function RichMessage({ text }: { text: string }) {
  return <div className={styles.richMessage}>{splitMarkdown(text).map((segment, index) => segment.kind === "text" ? <MarkdownText text={segment.value} key={index} /> : segment.language === "diff" ? <DiffBlock diff={segment.value} key={index} /> : segment.language === "mermaid" ? <MermaidBlock source={segment.value} key={index} /> : <CodeBlock code={segment.value} language={segment.language ?? "text"} key={index} />)}</div>;
}

function DraftStatus({ status }: { status: MessageDraft["status"] }) {
  const labels = { editable: "Ready to send", sending: "Sending…", sent: "Sent", discarded: "Discarded", failed: "Failed" } as const;
  return <span className={`${styles.cardStatus} ${styles[`cardStatus${status[0].toUpperCase()}${status.slice(1)}` as keyof typeof styles]}`}>{labels[status]}</span>;
}

function EmailDraftCard({ draft: initial, onResolve }: { draft: EmailDraft; onResolve: TranscriptCardViewProps["onResolveDraft"] }) {
  const [draft, setDraft] = useState(initial);
  const [expanded, setExpanded] = useState(initial.body.length < 420);
  useEffect(() => setDraft(initial), [initial]);
  const locked = draft.status !== "editable";
  return <article className={styles.transcriptCard}><header className={styles.transcriptCardHeader}><span className={styles.transcriptCardIcon}><Mail size={17} /></span><div><strong>New email</strong><DraftStatus status={draft.status} /></div></header>{draft.status === "sent" ? <p className={styles.sentSummary}>Sent to {draft.to[0]} — “{draft.subject}”</p> : <div className={styles.draftFields}>{draft.from ? <label><span>From</span><input disabled value={draft.from} /></label> : null}<label><span>To</span><input disabled={locked} value={draft.to.join(", ")} onChange={(event) => setDraft({ ...draft, to: event.target.value.split(",").map((item) => item.trim()).filter(Boolean) })} /></label><label><span>Subject</span><input disabled={locked} value={draft.subject} onChange={(event) => setDraft({ ...draft, subject: event.target.value })} /></label><label><span>Message</span><textarea className={!expanded ? styles.collapsedDraftBody : undefined} disabled={locked} value={draft.body} onChange={(event) => setDraft({ ...draft, body: event.target.value })} /></label>{draft.body.length >= 420 ? <button className={styles.textAction} type="button" onClick={() => setExpanded((value) => !value)}>{expanded ? <ChevronUp size={14} /> : <ChevronDown size={14} />}{expanded ? "Show less" : "Show more"}</button> : null}</div>}{draft.error ? <p className={styles.cardError}>{draft.error}</p> : null}{draft.status === "editable" ? <footer className={styles.cardActions}><button className={styles.secondaryCardButton} type="button" onClick={() => onResolve(draft, "discard")}><Trash2 size={15} /> Discard</button><button className={styles.primaryCardButton} type="button" onClick={() => onResolve(draft, "send")}><Send size={15} /> Send email</button></footer> : null}</article>;
}

function SlackDraftCard({ draft: initial, onResolve }: { draft: SlackDraft; onResolve: TranscriptCardViewProps["onResolveDraft"] }) {
  const [draft, setDraft] = useState(initial);
  useEffect(() => setDraft(initial), [initial]);
  const locked = draft.status !== "editable";
  return <article className={styles.transcriptCard}><header className={styles.transcriptCardHeader}><span className={styles.transcriptCardIcon}><MessageSquare size={17} /></span><div><strong>Slack message</strong><DraftStatus status={draft.status} /></div></header>{draft.status === "sent" ? <p className={styles.sentSummary}>Sent to {draft.target} — “{draft.body.split("\n")[0].slice(0, 80)}”</p> : <div className={styles.draftFields}>{draft.workspace ? <label><span>Workspace</span><input disabled value={draft.workspace} /></label> : null}<label><span>To</span><input disabled={locked} value={draft.target} onChange={(event) => setDraft({ ...draft, target: event.target.value })} /></label><label><span>Thread</span><input disabled={locked} value={draft.thread ?? ""} placeholder="New message" onChange={(event) => setDraft({ ...draft, thread: event.target.value || undefined })} /></label><label><span>Message</span><textarea disabled={locked} value={draft.body} onChange={(event) => setDraft({ ...draft, body: event.target.value })} /></label></div>}{draft.error ? <p className={styles.cardError}>{draft.error}</p> : null}{draft.status === "editable" ? <footer className={styles.cardActions}><button className={styles.secondaryCardButton} type="button" onClick={() => onResolve(draft, "discard")}><Trash2 size={15} /> Discard</button><button className={styles.primaryCardButton} type="button" onClick={() => onResolve(draft, "send")}><Send size={15} /> Send message</button></footer> : null}</article>;
}

function SecretRequestCard({ card, onProvide }: { card: Extract<TranscriptCard, { kind: "secretRequest" }>; onProvide: TranscriptCardViewProps["onProvideSecret"] }) {
  const [value, setValue] = useState("");
  return <article className={styles.transcriptCard}><header className={styles.transcriptCardHeader}><span className={styles.transcriptCardIcon}><KeyRound size={17} /></span><div><strong>{card.label}</strong><span className={styles.cardStatus}>{card.provided ? "Saved" : "Secret requested"}</span></div></header>{card.provided ? <p className={styles.sentSummary}>Saved securely and kept private.</p> : <><p className={styles.cardDescription}>{card.description}</p><input className={styles.secretInput} type="password" autoComplete="off" value={value} onChange={(event) => setValue(event.target.value)} placeholder={card.label} /><footer className={styles.cardActions}><span className={styles.privateHint}>Stored securely, never shown to your agent.</span><button className={styles.primaryCardButton} type="button" disabled={!value} onClick={() => { onProvide(card.requestId, value); setValue(""); }}><KeyRound size={15} /> Save securely</button></footer></>}</article>;
}

function ListenerConnectCard({ card, onConnect }: { card: Extract<TranscriptCard, { kind: "listenerConnect" }>; onConnect: TranscriptCardViewProps["onConnectListener"] }) {
  const copy = listenerCopy[card.platform];
  return <article className={styles.transcriptCard}><header className={styles.transcriptCardHeader}><span className={styles.transcriptCardIcon}><GitBranch size={17} /></span><div><strong>{card.connected ? `${copy[0]} connected` : `Connect ${copy[0]}`}</strong><span className={styles.cardStatus}>{card.pending ? "Checking connection status" : card.connected ? "Connected" : "Connection required"}</span></div></header><p className={styles.cardDescription}>{card.reason || copy[1]}</p>{!card.connected ? <footer className={styles.cardActions}><button className={styles.primaryCardButton} type="button" disabled={card.pending} onClick={() => onConnect(card.platform)}>{card.pending ? "Checking…" : "Connect"}</button></footer> : null}</article>;
}

function EventResultCard({ card }: { card: Extract<TranscriptCard, { kind: "event" }> }) {
  const event = card.event;
  const url = event.url ? safeHref(event.url) : null;
  return <article className={styles.eventCard}><header><span className={styles.eventSource}>{listenerCopy[event.source][0]}</span><span>{event.event}</span></header><h4>{event.title}</h4><p>{event.summary}</p>{event.actor ? <span className={styles.eventActor}>Triggered by {event.actor}</span> : null}{event.fields?.length ? <dl>{event.fields.map((field) => <div key={field.label}><dt>{field.label}</dt><dd>{field.value}</dd></div>)}</dl> : null}{url ? <a href={url} target="_blank" rel="noreferrer">Open event <ExternalLink size={14} /></a> : null}</article>;
}

function PdfCard({ card }: { card: Extract<TranscriptCard, { kind: "pdf" }> }) {
  const source = safeDocumentSource(
    card.url || (card.dataBase64 ? `data:application/pdf;base64,${card.dataBase64}` : undefined),
  );
  return <article className={styles.documentCard}><header><FileText size={17} /><strong>{card.name}</strong>{card.pageCount ? <span>{card.pageCount} pages</span> : null}{source ? <a href={source} target="_blank" rel="noreferrer"><ExternalLink size={14} /> Open</a> : null}</header>{source ? <object className={styles.pdfPreview} data={source} type="application/pdf"><a href={source}>Open PDF</a></object> : <p>PDF content is unavailable.</p>}</article>;
}


function MiniAppCard({ card }: { card: Extract<TranscriptCard, { kind: "miniApp" }> }) {
  const canOpen = typeof window !== "undefined" && typeof window.fabushi?.registerMiniAppDocument === "function";
  return <article className={styles.documentCard} data-testid="transcript-miniapp-card"><header><AppWindow size={17} /><strong>{card.name}</strong><span>Mini App</span></header><p>{card.description || "Runnable Fabushi Mini App artifact."}</p>{canOpen ? <footer className={styles.cardActions}><button className={styles.primaryCardButton} type="button" onClick={() => window.dispatchEvent(new CustomEvent("fabushi:open-generated-miniapp", { detail: card }))}><AppWindow size={15} /> 打开小程序</button></footer> : <p className={styles.cardDescription}>请在 Fabushi 桌面或移动应用中打开此产物。</p>}</article>;
}

function SpreadsheetCard({ card }: { card: Extract<TranscriptCard, { kind: "spreadsheet" }> }) {
  const [active, setActive] = useState(0);
  const sheet = card.sheets[active];
  return <article className={styles.documentCard}><header><FileSpreadsheet size={17} /><strong>{card.name}</strong><span>{card.sheets.length} sheet{card.sheets.length === 1 ? "" : "s"}</span></header><nav className={styles.sheetTabs}>{card.sheets.map((item, index) => <button className={index === active ? styles.sheetTabActive : undefined} type="button" key={item.name} onClick={() => setActive(index)}>{item.name}</button>)}</nav>{sheet ? <div className={styles.sheetTableWrap}><table className={styles.sheetTable}><tbody>{sheet.rows.slice(0, 200).map((row, rowIndex) => <tr key={rowIndex}>{row.slice(0, 100).map((cell, cellIndex) => rowIndex === 0 ? <th key={cellIndex}>{cell}</th> : <td key={cellIndex}>{cell}</td>)}</tr>)}</tbody></table></div> : <p>No spreadsheet rows.</p>}</article>;
}

export function TranscriptCardView({ entry, onResolveDraft, onProvideSecret, onConnectListener }: TranscriptCardViewProps) {
  const card = entry.card;
  if (card.kind === "emailDraft" && card.draft.kind === "email") return <EmailDraftCard draft={card.draft} onResolve={onResolveDraft} />;
  if (card.kind === "slackDraft" && card.draft.kind === "slack") return <SlackDraftCard draft={card.draft} onResolve={onResolveDraft} />;
  if (card.kind === "secretRequest") return <SecretRequestCard card={card} onProvide={onProvideSecret} />;
  if (card.kind === "listenerConnect") return <ListenerConnectCard card={card} onConnect={onConnectListener} />;
  if (card.kind === "event") return <EventResultCard card={card} />;
  if (card.kind === "pdf") return <PdfCard card={card} />;
  if (card.kind === "spreadsheet") return <SpreadsheetCard card={card} />;
  if (card.kind === "miniApp") return <MiniAppCard card={card} />;
  return null;
}
