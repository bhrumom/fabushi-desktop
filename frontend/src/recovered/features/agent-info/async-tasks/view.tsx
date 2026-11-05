import { useMemo, useSyncExternalStore } from "react";
import {
  formatAsyncTaskTime,
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

// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2784222 (L3n task row; SHA256 ef4e9831b65d39633f09c9ad0c083b98b7ebf52e3bb558182aee5bde31f876fa)
// @evidence src/app/dist/renderer/assets/index-UbX-y3il.js#byteOffset=2786671 (B3n async tasks panel; SHA256 ef4e9831b65d39633f09c9ad0c083b98b7ebf52e3bb558182aee5bde31f876fa)
// @evidence recovered/frontend/app/assets/index-UbX-y3il.js#byteOffset=3549518 (L3n task row; SHA256 80464803b50f478598080bdc1b91da3996c6b74168e2351ea26f620f2ec62ba5)
// @evidence recovered/frontend/app/assets/index-UbX-y3il.js#byteOffset=3552629 (B3n async tasks panel; SHA256 80464803b50f478598080bdc1b91da3996c6b74168e2351ea26f620f2ec62ba5)

export interface AsyncTasksPanelProps {
  readonly agentId: string;
  readonly agentName?: string;
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

export function AsyncTasksPanel({ agentId, agentName, onClose, provider, clock = DEFAULT_ASYNC_TASKS_CLOCK }: AsyncTasksPanelProps) {
  const handle = useMemo(() => provider.snapshotsFor(agentId), [agentId, provider]);
  const snapshot = useSyncExternalStore(handle.subscribe, handle.get, handle.get);
  const nowMs = useClockNow(clock);
  const { attachPanel, onHeaderPointerDown } = useMovablePanel();
  const name = agentName != null && agentName.length > 0 ? agentName : "Agent";
  const label = `Async tasks: ${name}`;
  const tasks = taskList(snapshot);
  return <aside aria-label={label} className="sand-async-tasks-panel sand-ixxii4 sand-1ng4z2i sand-n5hqff sand-8k05lb sand-78zum5 sand-dt5ytf sand-j6ak53 sand-w7nakj sand-1cnf4oa sand-ixl9f9 sand-mkeg23 sand-1y0btm7 sand-qz0629 sand-f1vpex sand-1jn3cnd sand-b3r6kr sand-lvsv26 sand-17ub312 sand-1aquc0h sand-1shwlz sand-a0mk88" ref={attachPanel} role="dialog">
    <header className="sand-async-tasks-panel__header sand-78zum5 sand-6s0dn4 sand-1qughib sand-167g77z sand-889kno sand-cicffo sand-1a8lsjc sand-zjhap9 sand-so031l sand-1q0q8m5 sand-17fyfba sand-1jm3nie sand-87ps6o sand-5ve5x3" onPointerDown={onHeaderPointerDown}>
      <div className="sand-async-tasks-panel__title sand-78zum5 sand-6s0dn4 sand-167g77z sand-euugli sand-1wd3ewq">
        <span aria-hidden="true" className="sand-2lah0s sand-4b2ntj" data-icon-name="clock" />
        <div className="sand-async-tasks-panel__title-text sand-78zum5 sand-dt5ytf sand-euugli"><strong className="sand-b3r6kr sand-lyipyv sand-uxw1ft">Async tasks</strong><span className="sand-b3r6kr sand-lyipyv sand-uxw1ft">{name}</span></div>
      </div>
      <button aria-label="Close async tasks" data-icon-name="close" onClick={onClose} type="button" />
    </header>
    <div aria-label={label} className="sand-async-tasks-panel__list sand-1iyjqo2 sand-s83m0k sand-dl72j9 sand-2lwn1j sand-9f619 sand-78zum5 sand-dt5ytf sand-195vfkc sand-1ruevkc" role="list">
      {tasks.length === 0
        ? <div className="sand-async-tasks-empty sand-1p5oq8j sand-nuq7ks sand-wxc41k sand-f18ygs sand-2b8uid">No async tasks in progress.</div>
        : tasks.map((task) => <AsyncTaskRow key={`${task.kind}:${task.id}`} nowMs={nowMs} task={task} />)}
    </div>
  </aside>;
}
