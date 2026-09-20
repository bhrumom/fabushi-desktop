import type { JsonSchema } from "./types";

export const FABUSHI_APP_SURFACE_VERSION = 1 as const;

export const FABUSHI_APP_TOOL_NAMES = Object.freeze({
  status: "fabushi.app.status",
  snapshot: "fabushi.app.snapshot",
  find: "fabushi.app.find",
  action: "fabushi.app.action",
  wait: "fabushi.app.wait",
  assert: "fabushi.app.assert",
});

export type FabushiAppToolName =
  (typeof FABUSHI_APP_TOOL_NAMES)[keyof typeof FABUSHI_APP_TOOL_NAMES];

export interface FabushiAppSurfaceToolDescriptor {
  name: FabushiAppToolName;
  title: string;
  description: string;
  inputSchema: JsonSchema;
  annotations: { readOnlyHint: boolean };
}

type InternalAppSurfaceToolDefinition = Omit<FabushiAppSurfaceToolDescriptor, "annotations"> & {
  operation: AppSurfaceOperation;
  readOnly: boolean;
};

export type AppSurfaceOperation = "status" | "snapshot" | "find" | "action" | "wait" | "assert";

export interface AppSurfaceInvocationOptions {
  signal?: AbortSignal;
}

export interface AppSurface {
  readonly version: typeof FABUSHI_APP_SURFACE_VERSION;
  readonly appId: string;
  call(
    operation: AppSurfaceOperation,
    input?: Record<string, unknown>,
    options?: AppSurfaceInvocationOptions,
  ): Promise<unknown>;
}

export interface FabushiAppSurfaceRegistry {
  readonly version: typeof FABUSHI_APP_SURFACE_VERSION;
  readonly appId: string;
  list(): FabushiAppSurfaceToolDescriptor[];
  call(name: FabushiAppToolName, input?: Record<string, unknown>): Promise<unknown>;
}

const emptyObjectSchema: JsonSchema = { type: "object", properties: {} };

const toolDefinitions: readonly InternalAppSurfaceToolDefinition[] = [
  {
    operation: "status" as const,
    name: FABUSHI_APP_TOOL_NAMES.status,
    title: "Fabushi app agent-surface status",
    description:
      "Report whether the active Fabushi application exposes its structured App MCP surface, including app identity, route, current screen and generation. Use this before controlling Fabushi.",
    inputSchema: emptyObjectSchema,
    readOnly: true,
  },
  {
    operation: "snapshot" as const,
    name: FABUSHI_APP_TOOL_NAMES.snapshot,
    title: "Read the Fabushi semantic UI",
    description:
      "Return a structured, redacted semantic snapshot of the active Fabushi UI: route, screen, generation and actionable elements. This is preferred over screenshots for Fabushi itself.",
    inputSchema: {
      type: "object",
      properties: {
        maxElements: { type: "integer", description: "Maximum semantic elements, from 1 to 500." },
        includeText: { type: "boolean", description: "Include bounded visible labels and text. Sensitive values remain redacted." },
      },
    },
    readOnly: true,
  },
  {
    operation: "find" as const,
    name: FABUSHI_APP_TOOL_NAMES.find,
    title: "Find Fabushi UI elements",
    description:
      "Find elements in the current Fabushi semantic surface by stable agent id, generation-bound ref, role, accessible name or visible text without image recognition.",
    inputSchema: {
      type: "object",
      properties: {
        agentId: { type: "string", description: "Stable data-agent-id, data-testid or DOM id." },
        ref: { type: "string", description: "Generation-bound element ref from a snapshot." },
        role: { type: "string", description: "ARIA/native semantic role." },
        name: { type: "string", description: "Accessible name substring." },
        text: { type: "string", description: "Visible text substring." },
        limit: { type: "integer", description: "Maximum matches, from 1 to 100." },
      },
    },
    readOnly: true,
  },
  {
    operation: "action" as const,
    name: FABUSHI_APP_TOOL_NAMES.action,
    title: "Operate the Fabushi semantic UI",
    description:
      "Perform one allowlisted action on a Fabushi semantic element. The current generation is mandatory for stale-reference protection. Password and credential fields are never writable through this tool.",
    inputSchema: {
      type: "object",
      properties: {
        generation: { type: "integer", description: "Exact generation returned by snapshot/find." },
        ref: { type: "string", description: "Generation-bound ref returned by snapshot/find." },
        agentId: { type: "string", description: "Stable agent id returned by snapshot/find." },
        action: {
          type: "string",
          enum: ["invoke", "focus", "setValue", "pressKey", "scroll", "selectOption", "toggle"],
        },
        value: { type: "string", description: "Text, key chord, select value or scroll direction/amount encoding required by the selected action." },
      },
      required: ["generation", "action"],
    },
    readOnly: false,
  },
  {
    operation: "wait" as const,
    name: FABUSHI_APP_TOOL_NAMES.wait,
    title: "Wait for Fabushi UI state",
    description:
      "Wait for a route, screen, element, name or text condition in the structured Fabushi UI. Timeouts are bounded and the operation is cancellable.",
    inputSchema: {
      type: "object",
      properties: {
        route: { type: "string" },
        screen: { type: "string" },
        agentId: { type: "string" },
        role: { type: "string" },
        name: { type: "string" },
        text: { type: "string" },
        state: { type: "string", enum: ["present", "absent", "enabled", "disabled", "visible", "hidden"] },
        timeoutMs: { type: "integer", description: "Bounded to 100-30000 ms." },
      },
    },
    readOnly: true,
  },
  {
    operation: "assert" as const,
    name: FABUSHI_APP_TOOL_NAMES.assert,
    title: "Assert Fabushi UI state",
    description:
      "Evaluate a deterministic assertion against the structured Fabushi UI and return observations suitable for CI evidence.",
    inputSchema: {
      type: "object",
      properties: {
        route: { type: "string" },
        screen: { type: "string" },
        agentId: { type: "string" },
        role: { type: "string" },
        name: { type: "string" },
        text: { type: "string" },
        state: { type: "string", enum: ["present", "absent", "enabled", "disabled", "visible", "hidden"] },
      },
    },
    readOnly: true,
  },
];
Object.freeze(toolDefinitions);

export function appSurfaceToolDefinitions(): FabushiAppSurfaceToolDescriptor[] {
  return toolDefinitions.map(({ operation: _operation, readOnly, ...tool }) => ({
    ...tool,
    annotations: { readOnlyHint: readOnly },
  }));
}

export function appSurfaceRegistry(surface: AppSurface): FabushiAppSurfaceRegistry {
  const byName = new Map(toolDefinitions.map((definition) => [definition.name, definition]));
  return Object.freeze({
    version: FABUSHI_APP_SURFACE_VERSION,
    appId: surface.appId,
    list: appSurfaceToolDefinitions,
    async call(name: FabushiAppToolName, input: Record<string, unknown> = {}) {
      const definition = byName.get(name);
      if (!definition) throw new Error(`Unknown Fabushi App MCP tool: ${name}`);
      return surface.call(definition.operation, input);
    },
  });
}
