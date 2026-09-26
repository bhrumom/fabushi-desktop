import { spawn, type ChildProcessWithoutNullStreams } from "node:child_process";
import { createInterface } from "node:readline";
import { join, resolve } from "node:path";
import { parentPort } from "electron";

type CarrierChannel =
  | "coordinator-control"
  | "coordinator-data"
  | "coordinator-main-data";

type CarrierEnvelope = {
  readonly channel: CarrierChannel;
  readonly frame: unknown;
};

type TransferPort = {
  postMessage(value: unknown): void;
  on(event: "message", listener: (event: { readonly data: unknown }) => void): void;
  on(event: "close", listener: () => void): void;
  start(): void;
  close(): void;
};

const CHANNELS: readonly CarrierChannel[] = [
  "coordinator-control",
  "coordinator-data",
  "coordinator-main-data",
];

function coordinatorExecutableName(): string {
  return process.platform === "win32"
    ? "mahayana-node-agent-coordinator.exe"
    : "mahayana-node-agent-coordinator";
}

function coordinatorExecutablePath(): string {
  const override = process.env.SAND_COORDINATOR_BIN;
  if (override != null && override.trim().length > 0) return override;
  if (process.env.SAND_PACKAGED === "1") {
    return join(process.resourcesPath, "bin", coordinatorExecutableName());
  }
  return resolve(__dirname, "..", "..", "resources", "bin", coordinatorExecutableName());
}

function writeEnvelope(
  child: ChildProcessWithoutNullStreams,
  channel: CarrierChannel,
  frame: unknown,
): void {
  child.stdin.write(JSON.stringify({ channel, frame } satisfies CarrierEnvelope) + "\n");
}

function isEnvelope(value: unknown): value is CarrierEnvelope {
  if (value == null || typeof value !== "object" || Array.isArray(value)) return false;
  const record = value as Record<string, unknown>;
  return CHANNELS.includes(record.channel as CarrierChannel) && "frame" in record;
}

if (parentPort == null) {
  throw new Error("Rust Coordinator carrier requires Electron utilityProcess parentPort.");
}

let child: ChildProcessWithoutNullStreams | undefined;
let ports: readonly TransferPort[] = [];
let closed = false;

const close = (code = 0): void => {
  if (closed) return;
  closed = true;
  for (const port of ports) {
    try { port.close(); } catch {}
  }
  if (child != null && child.exitCode == null && child.signalCode == null) {
    try { child.kill(); } catch {}
  }
  process.exitCode = code;
};

parentPort.once("message", (event: { readonly data: unknown; readonly ports?: readonly TransferPort[] }) => {
  const bootstrap = event.data as { readonly bootstrap?: unknown } | null;
  const transferred = event.ports ?? [];
  if (bootstrap == null || typeof bootstrap !== "object" || bootstrap.bootstrap == null) {
    throw new Error("Rust Coordinator carrier received no bootstrap payload.");
  }
  if (transferred.length !== CHANNELS.length) {
    throw new Error(`Rust Coordinator carrier requires ${CHANNELS.length} transferred ports.`);
  }

  ports = transferred;
  child = spawn(
    coordinatorExecutablePath(),
    [`--bootstrap=${JSON.stringify(bootstrap.bootstrap)}`],
    {
      stdio: ["pipe", "pipe", "pipe"],
      env: process.env,
      windowsHide: true,
    },
  );

  transferred.forEach((port, index) => {
    const channel = CHANNELS[index]!;
    port.on("message", (message) => writeEnvelope(child!, channel, message.data));
    port.on("close", () => {
      if (transferred.every((candidate) => candidate === port || closed)) close(0);
    });
    port.start();
  });

  const output = createInterface({ input: child.stdout });
  output.on("line", (line) => {
    if (line.trim().length === 0) return;
    let parsed: unknown;
    try {
      parsed = JSON.parse(line);
    } catch {
      console.error("[mahayana-coordinator-carrier] invalid coordinator stdout", line);
      return;
    }
    if (!isEnvelope(parsed)) {
      console.error("[mahayana-coordinator-carrier] invalid coordinator envelope", parsed);
      return;
    }
    const index = CHANNELS.indexOf(parsed.channel);
    const port = ports[index];
    if (port != null) port.postMessage(parsed.frame);
  });

  child.stderr.on("data", (chunk) => {
    process.stderr.write(chunk);
  });
  child.on("error", (error) => {
    console.error("[mahayana-coordinator-carrier] coordinator process error", error);
    close(1);
  });
  child.on("exit", (code, signal) => {
    if (signal != null) {
      console.error("[mahayana-coordinator-carrier] coordinator exited by signal", signal);
    }
    close(code ?? (signal == null ? 0 : 1));
  });
});

process.on("exit", () => {
  if (child != null && child.exitCode == null && child.signalCode == null) {
    try { child.kill(); } catch {}
  }
});
