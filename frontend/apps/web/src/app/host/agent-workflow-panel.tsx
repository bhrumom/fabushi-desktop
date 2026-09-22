"use client";

import React, { useMemo, useState, type FormEvent } from "react";
import type { WorkflowSummary, WorkflowTrigger } from "../../lib/mahayana-host/contracts";
import styles from "./host.module.css";

type WorkflowDraft = {
  id?: string;
  name: string;
  description: string;
  body: string;
  scheduled: boolean;
  schedule: string;
  scheduleEnabled: boolean;
  sourceRef: string;
};

const blankDraft: WorkflowDraft = {
  name: "",
  description: "",
  body: "",
  scheduled: false,
  schedule: "",
  scheduleEnabled: true,
  sourceRef: "",
};

function draftFromWorkflow(workflow: WorkflowSummary): WorkflowDraft {
  return {
    id: workflow.id,
    name: workflow.name,
    description: workflow.description,
    body: workflow.body,
    scheduled: Boolean(workflow.trigger),
    schedule: workflow.trigger?.schedule ?? "",
    scheduleEnabled: workflow.trigger?.isEnabled ?? true,
    sourceRef: workflow.sourceRef ?? "",
  };
}

function formatRunTime(value?: number): string {
  if (!value) return "";
  try {
    return new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" }).format(value);
  } catch {
    return "";
  }
}

export function AgentWorkflowPanel({
  agentId,
  workflows,
  onRefresh,
  onSave,
  onSetEnabled,
  onRun,
  onDelete,
  onImportMarkdown,
  onImportLiveSource,
}: {
  agentId: string;
  workflows: WorkflowSummary[];
  onRefresh: () => void;
  onSave: (draft: {
    id?: string;
    name: string;
    description: string;
    body: string;
    trigger?: WorkflowTrigger;
    sourceRef?: string;
  }) => Promise<void> | void;
  onSetEnabled: (id: string, enabled: boolean) => Promise<void> | void;
  onRun: (id: string) => Promise<void> | void;
  onDelete: (id: string, name: string) => Promise<void> | void;
  onImportMarkdown: (markdown: string, fallbackName?: string) => Promise<void> | void;
  onImportLiveSource: (source: string, fallbackName?: string) => Promise<void> | void;
}) {
  const [editorOpen, setEditorOpen] = useState(false);
  const [draft, setDraft] = useState<WorkflowDraft>(blankDraft);
  const [importMode, setImportMode] = useState<"none" | "markdown" | "live">("none");
  const [importText, setImportText] = useState("");
  const [importName, setImportName] = useState("");
  const sorted = useMemo(
    () => [...workflows].sort((a, b) => Number(Boolean(a.trigger)) - Number(Boolean(b.trigger)) || a.name.localeCompare(b.name)),
    [workflows],
  );

  const submit = async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const trigger = draft.scheduled
      ? { schedule: draft.schedule.trim(), isEnabled: draft.scheduleEnabled }
      : undefined;
    if (!draft.name.trim() || !draft.body.trim() || (draft.scheduled && !trigger?.schedule)) return;
    await onSave({
      id: draft.id,
      name: draft.name.trim(),
      description: draft.description.trim(),
      body: draft.body,
      trigger,
      sourceRef: draft.sourceRef.trim() || undefined,
    });
    setDraft(blankDraft);
    setEditorOpen(false);
  };

  const doImport = async () => {
    const value = importText.trim();
    if (!value) return;
    if (importMode === "markdown") await onImportMarkdown(value, importName.trim() || undefined);
    if (importMode === "live") await onImportLiveSource(value, importName.trim() || undefined);
    setImportText("");
    setImportName("");
    setImportMode("none");
  };

  return (
    <section className={styles.agentWorkflowPanel} data-agent-id={agentId}>
      <header>
        <div>
          <strong>Workflows</strong>
          <small>{workflows.length} 个 · SKILL.md 全局库，每个 Bot 独立启用/禁用</small>
        </div>
        <div>
          <button type="button" onClick={onRefresh}>刷新</button>
          <button type="button" onClick={() => { setDraft(blankDraft); setEditorOpen(true); }}>＋ 新建</button>
        </div>
      </header>

      <div className={styles.workflowImportActions}>
        <button type="button" onClick={() => setImportMode(importMode === "markdown" ? "none" : "markdown")}>导入 Markdown</button>
        <button type="button" onClick={() => setImportMode(importMode === "live" ? "none" : "live")}>Live Source</button>
      </div>

      {importMode !== "none" ? (
        <div className={styles.workflowImporter}>
          <input value={importName} onChange={(event) => setImportName(event.target.value)} placeholder="可选名称" />
          {importMode === "markdown" ? (
            <textarea rows={7} value={importText} onChange={(event) => setImportText(event.target.value)} placeholder={'---\nname: My Workflow\n---\n# Instructions\n...'} />
          ) : (
            <input value={importText} onChange={(event) => setImportText(event.target.value)} placeholder="文件路径或 URL" />
          )}
          <footer><button type="button" onClick={() => setImportMode("none")}>取消</button><button type="button" disabled={!importText.trim()} onClick={() => void doImport()}>导入</button></footer>
        </div>
      ) : null}

      {editorOpen ? (
        <form className={styles.workflowEditor} onSubmit={(event) => void submit(event)}>
          <header><strong>{draft.id ? "编辑 Workflow" : "新建 Workflow"}</strong><button type="button" onClick={() => setEditorOpen(false)}>×</button></header>
          <label><span>名称</span><input required maxLength={80} value={draft.name} onChange={(event) => setDraft({ ...draft, name: event.target.value })} /></label>
          <label><span>描述</span><input maxLength={1536} value={draft.description} onChange={(event) => setDraft({ ...draft, description: event.target.value })} /></label>
          <label><span>指令 / SKILL.md body</span><textarea required rows={9} maxLength={100000} value={draft.body} onChange={(event) => setDraft({ ...draft, body: event.target.value })} /></label>
          <label><span>Live Source</span><input value={draft.sourceRef} onChange={(event) => setDraft({ ...draft, sourceRef: event.target.value })} placeholder="可选；保存在 metadata.source" /></label>
          <label className={styles.workflowScheduleToggle}><span><strong>定时触发</strong><small>启用后会同步为可独立运行的 Automation</small></span><input type="checkbox" checked={draft.scheduled} onChange={(event) => setDraft({ ...draft, scheduled: event.target.checked })} /></label>
          {draft.scheduled ? (
            <div className={styles.workflowScheduleFields}>
              <label><span>Schedule</span><input required value={draft.schedule} onChange={(event) => setDraft({ ...draft, schedule: event.target.value })} placeholder="@daily / every 1h / cron expression" /></label>
              <label><span>Enabled</span><input type="checkbox" checked={draft.scheduleEnabled} onChange={(event) => setDraft({ ...draft, scheduleEnabled: event.target.checked })} /></label>
            </div>
          ) : null}
          <footer><button type="button" onClick={() => setEditorOpen(false)}>取消</button><button type="submit">保存</button></footer>
        </form>
      ) : null}

      <div className={styles.workflowList}>
        {sorted.map((workflow) => (
          <article key={`${workflow.source}-${workflow.id}`} data-disabled={!workflow.isEnabledForAgent}>
            <div className={styles.workflowIcon} data-source={workflow.source}>{workflow.trigger ? "⏱" : "⌘"}</div>
            <div className={styles.workflowCopy}>
              <strong>{workflow.name}</strong>
              <p>{workflow.description || (workflow.trigger ? "Scheduled workflow" : "Reusable workflow")}</p>
              <small>
                {workflow.trigger ? `${workflow.trigger.schedule}${workflow.nextRunAt ? ` · next ${formatRunTime(workflow.nextRunAt)}` : ""}` : workflow.sourceRef || workflow.filePath}
              </small>
            </div>
            <div className={styles.workflowActions}>
              <label title="为当前 Bot 启用">
                <input type="checkbox" checked={workflow.isEnabledForAgent} onChange={(event) => void onSetEnabled(workflow.id, event.target.checked)} />
              </label>
              <button type="button" disabled={!workflow.isEnabledForAgent} onClick={() => void onRun(workflow.id)}>运行</button>
              <button type="button" onClick={() => { setDraft(draftFromWorkflow(workflow)); setEditorOpen(true); }}>编辑</button>
              <button type="button" className={styles.dangerAction} onClick={() => void onDelete(workflow.id, workflow.name)}>删除</button>
            </div>
          </article>
        ))}
        {!sorted.length ? <p className={styles.workflowEmpty}>还没有 Workflow。普通 Workflow 会进入 Bot 的可发现目录；带 Schedule 的 Workflow 会成为 Automation。</p> : null}
      </div>
    </section>
  );
}
