import { useMemo, useState, useSyncExternalStore } from "react";
import {
  formatAsyncTaskTime,
  GLOBAL_ASYNC_TASKS_SCOPE_ID,
  groupAsyncTasksByLineage,
  type AsyncTask,
  type AsyncTaskStatus,
  type AsyncTasksProvider,
  type AsyncTasksSnapshot,
} from "./provider";
import {
  createStableAsyncTasksClock,
  DEFAULT_ASYNC_TASKS_CLOCK,
  type AsyncTasksClockInput,
} from "./clock";
import { useMovablePanel } from "../../../ui/movable-panel";

// Upstream Source Provenance (Blueprint 05 §1 & §4):
// - OpenClaw: src/agents/subagent-registry.ts & ui/src/pages/tasks/ (Global Tasks & Subagent Swarm tree)
// - Telegram Desktop: Telegram/SourceFiles/data/data_todo_list.h
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2784222 (L3n task row; SHA256 ef4e9831b65d39633f09c9ad0c083b98b7ebf52e3bb558182aee5bde31f876fa)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2786671 (B3n async tasks panel; SHA256 ef4e9831b65d39633f09c9ad0c083b98b7ebf52e3bb558182aee5bde31f876fa)
// @evidence recovered/frontend/app/assets/index-UbX-y3il.js#byteOffset=3549518 (L3n task row; SHA256 80464803b50f478598080bdc1b91da3996c6b74168e2351ea26f620f2ec62ba5)
// @evidence recovered/frontend/app/assets/index-UbX-y3il.js#byteOffset=3552629 (B3n async tasks panel; SHA256 80464803b50f478598080bdc1b91da3996c6b74168e2351ea26f620f2ec62ba5)

export interface AsyncTasksPanelProps {
  readonly agentId?: string | null;
  readonly agentName?: string;
  readonly mode?: "conversation" | "global";
  readonly onClose: () => void;
  readonly provider: AsyncTasksProvider;
  readonly clock?: AsyncTasksClockInput;
}

function taskKindIcon(kind: AsyncTask["kind"]): string {
  switch (kind) {
    case "subagent": return "agent";
    case "shell": return "terminal-rectangle";
    case "cloud-agent": return "cloud-arrow-up";
  }
}

function taskKindLabel(kind: AsyncTask["kind"]): string {
  switch (kind) {
    case "subagent": return "Subagent";
    case "shell": return "Shell";
    case "cloud-agent": return "Cloud agent";
  }
}

export function taskStatusLabel(status: AsyncTaskStatus): string {
  switch (status) {
    case "pending": return "待开始";
    case "running": return "执行中";
    case "waiting_auth": return "等待授权";
    case "waiting_connection": return "等待连接";
    case "completed": return "已完成";
    case "failed": return "失败";
    case "cancelled": return "已取消";
    default: return status;
  }
}

function taskMeta(task: AsyncTask): string {
  const parts: string[] = [taskKindLabel(task.kind)];
  const status = taskStatusLabel(task.status);
  if (status) parts.push(status);
  if (typeof task.attempt === "number" && task.attempt > 1) {
    parts.push(`第 ${task.attempt} 次尝试`);
  }
  const detail = task.detail?.trim() ?? "";
  if (detail.length > 0) parts.push(detail);
  return parts.join(" · ");
}

function useClockNow(clock: AsyncTasksClockInput): number {
  const stableClock = useMemo(() => createStableAsyncTasksClock(clock), [clock]);
  return useSyncExternalStore(stableClock.subscribe, stableClock.now, stableClock.now);
}

function AsyncTaskRow({ task, nowMs }: { readonly task: AsyncTask; readonly nowMs: number }) {
  const started = new Date(task.startedAtMs).toLocaleString();
  const statusText = taskStatusLabel(task.status);
  const attemptText = typeof task.attempt === "number" && task.attempt > 1 ? ` (第 ${task.attempt} 次尝试)` : "";
  const errorText = task.error ? ` — 错误: ${task.error}` : "";
  const title = `${task.id} [${statusText}] — started ${started}${attemptText}${errorText}`;

  return <div
    className="sand-async-task sand-78zum5 sand-6s0dn4 sand-167g77z sand-m7lytj sand-y13l1i sand-1ykpatu sand-163pfp sand-ur7f20"
    data-kind={task.kind}
    data-status={task.status}
    role="listitem"
    title={title}
  >
    <span
      aria-label={statusText}
      className="sand-async-task__status sand-2lah0s sand-1v4s8kt sand-ols6we sand-149ho13 sand-zqwn8b"
      data-status={task.status}
      role="status"
    >
      {statusText}
    </span>
    <span aria-hidden="true" className="sand-async-task__icon sand-2lah0s sand-4b2ntj" data-icon-name={taskKindIcon(task.kind)} />
    <span className="sand-async-task__body sand-78zum5 sand-dt5ytf sand-1iyjqo2 sand-s83m0k sand-dl72j9 sand-euugli sand-12mrbbr">
      <span className="sand-async-task__label sand-b3r6kr sand-lyipyv sand-uxw1ft sand-18hujpe sand-1wd3ewq">{task.label}</span>
      <span className="sand-async-task__meta sand-b3r6kr sand-lyipyv sand-uxw1ft sand-vzzrjs sand-4b2ntj">{taskMeta(task)}</span>
      {(task.status === "failed" || Boolean(task.error)) && (
        <span
          className="sand-async-task__error sand-b3r6kr sand-lyipyv sand-uxw1ft sand-vzzrjs"
          style={{ color: "var(--fabushi-status-failed, #ef4444)", fontSize: "11.5px", marginTop: "2px" }}
        >
          {task.error ? `错误: ${task.error}` : "任务执行失败"}
        </span>
      )}
    </span>
    <span className="sand-async-task__time sand-2lah0s sand-vzzrjs sand-4b2ntj sand-ss6m8b">{formatAsyncTaskTime(task.startedAtMs, nowMs)}</span>
  </div>;
}

function taskList(snapshot: AsyncTasksSnapshot): readonly AsyncTask[] {
  if (snapshot.status === "ready") return snapshot.value;
  if (snapshot.status === "failed" && "previous" in snapshot) return snapshot.previous;
  return [];
}

export function AsyncTasksPanel({
  agentId,
  agentName,
  mode = "conversation",
  onClose,
  provider,
  clock = DEFAULT_ASYNC_TASKS_CLOCK,
}: AsyncTasksPanelProps) {
  const resolvedAgentId = agentId != null && agentId.trim().length > 0 ? agentId : GLOBAL_ASYNC_TASKS_SCOPE_ID;
  const handle = useMemo(() => provider.snapshotsFor(resolvedAgentId), [resolvedAgentId, provider]);
  const snapshot = useSyncExternalStore(handle.subscribe, handle.get, handle.get);
  const nowMs = useClockNow(clock);
  const [statusFilter, setStatusFilter] = useState<"all" | "waiting_auth" | "running" | "completed" | "failed">("all");
  const { attachPanel, onHeaderPointerDown } = useMovablePanel();
  const isGlobal = mode === "global" || resolvedAgentId === GLOBAL_ASYNC_TASKS_SCOPE_ID;
  const name = isGlobal
    ? "全局任务与自动化中心 (All Conversations)"
    : agentName != null && agentName.length > 0
      ? agentName
      : "Agent";
  const label = `Async tasks: ${name}`;
  const allTasks = taskList(snapshot);
  const filteredTasks = useMemo(() => {
    if (statusFilter === "all") return allTasks;
    return allTasks.filter((t) => t.status === statusFilter);
  }, [allTasks, statusFilter]);
  const lineageNodes = useMemo(() => groupAsyncTasksByLineage(filteredTasks), [filteredTasks]);

  return <aside aria-label={label} className="sand-async-tasks-panel sand-ixxii4 sand-1ng4z2i sand-n5hqff sand-8k05lb sand-78zum5 sand-dt5ytf sand-j6ak53 sand-w7nakj sand-1cnf4oa sand-ixl9f9 sand-mkeg23 sand-1y0btm7 sand-qz0629 sand-f1vpex sand-1jn3cnd sand-b3r6kr sand-lvsv26 sand-17ub312 sand-1aquc0h sand-1shwlz sand-a0mk88" ref={attachPanel} role="dialog">
    <header className="sand-async-tasks-panel__header sand-78zum5 sand-6s0dn4 sand-1qughib sand-167g77z sand-889kno sand-cicffo sand-1a8lsjc sand-zjhap9 sand-so031l sand-1q0q8m5 sand-17fyfba sand-1jm3nie sand-87ps6o sand-5ve5x3" onPointerDown={onHeaderPointerDown}>
      <div className="sand-async-tasks-panel__title sand-78zum5 sand-6s0dn4 sand-167g77z sand-euugli sand-1wd3ewq">
        <span aria-hidden="true" className="sand-2lah0s sand-4b2ntj" data-icon-name="clock" />
        <div className="sand-async-tasks-panel__title-text sand-78zum5 sand-dt5ytf sand-euugli"><strong className="sand-b3r6kr sand-lyipyv sand-uxw1ft">Async tasks</strong><span className="sand-b3r6kr sand-lyipyv sand-uxw1ft">{name}</span></div>
      </div>
      <button aria-label="Close async tasks" data-icon-name="close" onClick={onClose} type="button" />
    </header>
    {allTasks.length > 0 && (
      <div
        role="tablist"
        aria-label="Task status filter"
        style={{ display: "flex", gap: "4px", padding: "6px 12px", borderBottom: "1px solid var(--sand-border-weak, rgba(128,128,128,0.2))", flexWrap: "wrap" }}
      >
        {([
          ["all", "全部"],
          ["waiting_auth", "待授权"],
          ["running", "执行中"],
          ["completed", "已完成"],
          ["failed", "失败"],
        ] as const).map(([key, text]) => (
          <button
            key={key}
            type="button"
            role="tab"
            aria-selected={statusFilter === key}
            onClick={() => setStatusFilter(key)}
            style={{
              padding: "2px 8px",
              borderRadius: "999px",
              fontSize: "11px",
              border: "1px solid var(--sand-border-weak, rgba(128,128,128,0.25))",
              background: statusFilter === key ? "var(--fabushi-accent, #0a84ff)" : "transparent",
              color: statusFilter === key ? "#fff" : "inherit",
              cursor: "pointer",
            }}
          >
            {text}
          </button>
        ))}
      </div>
    )}
    <div aria-label={label} className="sand-async-tasks-panel__list sand-1iyjqo2 sand-s83m0k sand-dl72j9 sand-2lwn1j sand-9f619 sand-78zum5 sand-dt5ytf sand-195vfkc sand-1ruevkc" role="list">
      {lineageNodes.length === 0
        ? <div className="sand-async-tasks-empty sand-1p5oq8j sand-nuq7ks sand-wxc41k sand-f18ygs sand-2b8uid">No async tasks in progress.</div>
        : lineageNodes.map((node) => (
            <div key={`${node.task.kind}:${node.task.id}`} style={{ display: "flex", flexDirection: "column" }}>
              <AsyncTaskRow nowMs={nowMs} task={node.task} />
              {node.children.length > 0 && (
                <div style={{ paddingLeft: "18px", borderLeft: "2px solid var(--sand-border-weak, rgba(128,128,128,0.2))", marginLeft: "12px" }}>
                  {node.children.map((child) => (
                    <AsyncTaskRow key={`${child.kind}:${child.id}`} nowMs={nowMs} task={child} />
                  ))}
                </div>
              )}
            </div>
          ))}
    </div>
  </aside>;
}
