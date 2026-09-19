import crypto from "node:crypto";
import os from "node:os";
import process from "node:process";
import { BasePromptBuilder, BasePromptExecutor } from "../../reference/grok-bot-0.18/source/packages/chat-inference/base.js";
import { createStringResult } from "../../reference/grok-bot-0.18/source/packages/chat-inference/prompt-executor.js";
import { createContext } from "../../reference/grok-bot-0.18/source/packages/context/core.js";
import { InMemoryBlobStore } from "../../reference/grok-bot-0.18/source/packages/agent-kv/blob-store.js";
import { ToolSetHandle } from "../../reference/grok-bot-0.18/source/packages/agent/tools/core.js";
import { PrivacyMode } from "../../reference/grok-bot-0.18/source/packages/redaction/privacy-mode.js";
import {
  ConversationAction,
  ConversationStateStructure,
  UserMessage,
  UserMessageAction,
} from "../../reference/grok-bot-0.18/source/packages/proto/generated/agent/v1/agent_pb.js";
import {
  RequestContext,
  RequestContextEnv,
  RequestContextResult,
  RequestContextSuccess,
} from "../../reference/grok-bot-0.18/source/packages/proto/generated/agent/v1/request_context_exec_pb.js";
import { requestContextExecutorResource } from "../../reference/grok-bot-0.18/source/packages/agent-exec/request-context.js";
import {
  createSandAgentStaticConfig,
  createTurnAgentForRun,
  createTurnAgentStreamStart,
  createTurnToolSession,
} from "../../reference/grok-bot-0.18/source/host/runner/turn-agent-composition.js";

type Loose = Record<string, any>;
type TransportResult = {
  offline?: boolean;
  message?: {
    content?: string | null;
    tool_calls?: Array<{
      id?: string;
      type?: string;
      function?: { name?: string; arguments?: string };
    }>;
  };
  usage?: Loose;
};
export type LocalModelTransport = (
  messages: readonly Loose[],
  tools: readonly Loose[],
  signal: AbortSignal,
  onDelta?: (delta: string, aggregate: string) => void,
) => Promise<TransportResult>;

export interface LocalToolDefinition {
  readonly type?: string;
  readonly function: {
    readonly name: string;
    readonly description?: string;
    readonly parameters?: Loose;
  };
}
export interface LocalToolExecution {
  readonly text?: string;
  readonly display?: unknown;
}
export type LocalToolExecutor = (
  name: string,
  args: Loose,
  options: { readonly signal: AbortSignal; readonly toolCallId: string },
) => Promise<LocalToolExecution | string | null | undefined>;

class AsyncEventQueue<T> implements AsyncIterable<T> {
  #items: T[] = [];
  #waiters: Array<(value: IteratorResult<T>) => void> = [];
  #closed = false;
  push(value: T): void {
    if (this.#closed) return;
    const waiter = this.#waiters.shift();
    if (waiter) waiter({ value, done: false });
    else this.#items.push(value);
  }
  close(): void {
    if (this.#closed) return;
    this.#closed = true;
    for (const waiter of this.#waiters.splice(0)) waiter({ value: undefined as T, done: true });
  }
  [Symbol.asyncIterator](): AsyncIterator<T> {
    return {
      next: async () => {
        const value = this.#items.shift();
        if (value !== undefined) return { value, done: false };
        if (this.#closed) return { value: undefined as T, done: true };
        return await new Promise<IteratorResult<T>>(resolve => this.#waiters.push(resolve));
      },
    };
  }
}

function usageShape(value: Loose | undefined) {
  const inputTokens = Number(value?.inputTokens ?? value?.prompt_tokens ?? value?.input_tokens ?? 0) || 0;
  const outputTokens = Number(value?.outputTokens ?? value?.completion_tokens ?? value?.output_tokens ?? 0) || 0;
  const cacheReadTokens = Number(value?.cacheReadTokens ?? value?.prompt_tokens_details?.cached_tokens ?? 0) || 0;
  const cacheWriteTokens = Number(value?.cacheWriteTokens ?? value?.prompt_tokens_details?.cache_creation_tokens ?? 0) || 0;
  return {
    promptTokens: inputTokens,
    completionTokens: outputTokens,
    totalTokens: inputTokens + outputTokens,
    inputTokens,
    outputTokens,
    cacheReadTokens,
    cacheWriteTokens,
    maxTokens: Number(value?.maxTokens ?? 200_000) || 200_000,
  };
}

function toOpenAiMessages(messages: readonly Loose[]): Loose[] {
  return messages.map(message => {
    if (typeof message?.content === "string") return { ...message };
    if (!Array.isArray(message?.content)) return { ...message, content: String(message?.content ?? "") };
    if (message.role === "assistant") {
      const text = message.content.filter((part: Loose) => part?.type === "text").map((part: Loose) => String(part.text ?? "")).join("");
      const toolCalls = message.content.filter((part: Loose) => part?.type === "tool-call").map((part: Loose) => ({
        id: String(part.toolCallId ?? crypto.randomUUID()),
        type: "function",
        function: { name: String(part.toolName ?? "tool"), arguments: JSON.stringify(part.args ?? {}) },
      }));
      return { role: "assistant", content: text || null, ...(toolCalls.length ? { tool_calls: toolCalls } : {}) };
    }
    if (message.role === "tool") {
      const result = message.content.find((part: Loose) => part?.type === "tool-result");
      return {
        role: "tool",
        tool_call_id: String(result?.toolCallId ?? message.id ?? ""),
        content: typeof result?.result === "string" ? result.result : JSON.stringify(result?.result ?? {}),
      };
    }
    const text = message.content.map((part: Loose) => typeof part?.text === "string" ? part.text : "").join("");
    return { ...message, content: text };
  });
}

function toOpenAiTools(tools: readonly Loose[]): Loose[] {
  return tools.map(tool => ({
    type: "function",
    function: {
      name: String(tool.name),
      description: String(tool.description ?? ""),
      parameters: tool.parameters && typeof tool.parameters === "object" ? tool.parameters : { type: "object", properties: {} },
    },
  }));
}

class FabushiPromptExecutor extends BasePromptExecutor<Loose> {
  constructor(
    private readonly transport: LocalModelTransport,
    initialMessages?: readonly Loose[],
  ) {
    super(new BasePromptBuilder<Loose>(initialMessages));
  }

  stream(ctx: { signal: AbortSignal }, invocationId = crypto.randomUUID(), tools: readonly Loose[] = []) {
    const queue = new AsyncEventQueue<Loose>();
    let streamedText = "";
    const transportPromise = this.transport(
      toOpenAiMessages(this.getMessages()),
      toOpenAiTools(tools),
      ctx.signal,
      (delta, aggregate) => {
        streamedText = aggregate;
        if (delta) queue.push({ type: "text-delta", textDelta: delta });
      },
    );
    const response = transportPromise.then(result => {
      if (result.offline) {
        return {
          id: invocationId,
          timestamp: new Date(),
          modelId: "fabushi-offline",
          messages: [{ role: "assistant", content: [{ type: "text", text: "Agent inference is not configured on this Mac." }] }],
        };
      }
      const message = result.message ?? {};
      const content: Loose[] = [];
      if (typeof message.content === "string" && message.content.length > 0) content.push({ type: "text", text: message.content });
      for (const call of message.tool_calls ?? []) {
        let args: unknown = {};
        try { args = JSON.parse(call.function?.arguments ?? "{}"); } catch { args = call.function?.arguments ?? "{}"; }
        content.push({
          type: "tool-call",
          toolCallId: String(call.id ?? crypto.randomUUID()),
          toolName: String(call.function?.name ?? "tool"),
          args,
        });
      }
      return {
        id: invocationId,
        timestamp: new Date(),
        modelId: String(process.env.FABUSHI_AGENT_MODEL || "gpt-5.6"),
        messages: [{ role: "assistant", content }],
      };
    });

    void transportPromise.then(result => {
      if (result.offline) {
        queue.push({ type: "text-delta", textDelta: "Agent inference is not configured on this Mac." });
      } else {
        const message = result.message ?? {};
        if (typeof message.content === "string" && message.content.length > 0 && streamedText.length === 0) {
          queue.push({ type: "text-delta", textDelta: message.content });
        }
        for (const call of message.tool_calls ?? []) {
          const toolCallId = String(call.id ?? crypto.randomUUID());
          const toolName = String(call.function?.name ?? "tool");
          const argsText = String(call.function?.arguments ?? "{}");
          let args: unknown = {};
          try { args = JSON.parse(argsText); } catch { args = argsText; }
          queue.push({ type: "tool-call-streaming-start", toolCallId, toolName });
          queue.push({ type: "tool-call-delta", toolCallId, toolName, argsTextDelta: argsText });
          queue.push({ type: "tool-call", toolCallId, toolName, args });
        }
      }
      const usage = usageShape(result.usage);
      queue.push({
        type: "finish",
        finishReason: "stop",
        usage,
        logprobs: undefined,
        response: { id: invocationId, timestamp: new Date(), modelId: String(process.env.FABUSHI_AGENT_MODEL || "gpt-5.6") },
      });
      queue.close();
    }, error => {
      queue.push({ type: "error", error });
      queue.close();
    });

    const usage = transportPromise.then(result => usageShape(result.usage));
    return {
      fullStream: queue,
      response,
      usage,
      extendedUsage: usage.then(value => ({
        inputTokens: value.inputTokens,
        outputTokens: value.outputTokens,
        cacheReadTokens: value.cacheReadTokens,
        cacheWriteTokens: value.cacheWriteTokens,
        maxTokens: value.maxTokens,
      })),
      providerMetadata: Promise.resolve(undefined),
      invocationId: Promise.resolve(invocationId),
    };
  }
}

class LocalToolResult {
  constructor(readonly text: string, readonly display?: unknown, readonly isError = false) {}
  toJson() { return { text: this.text, display: this.display, isError: this.isError }; }
}

function makeTool(definition: LocalToolDefinition, executeTool: LocalToolExecutor): Loose {
  const name = String(definition.function.name);
  const identifier = name.replace(/[^A-Za-z0-9]+/g, "_").toUpperCase() || "LOCAL_TOOL";
  return {
    toolIdentifier: identifier,
    name,
    description: String(definition.function.description ?? ""),
    parameters: definition.function.parameters ?? { type: "object", properties: {} },
    async execute(ctx: { signal: AbortSignal }, _interaction: unknown, argsStream: AsyncIterable<string>, meta: Loose) {
      let raw = "";
      for await (const chunk of argsStream) raw += chunk;
      let args: Loose = {};
      try { args = raw.trim() ? JSON.parse(raw) : {}; } catch { args = {}; }
      const value = await executeTool(name, args, { signal: ctx.signal, toolCallId: String(meta?.toolCallId ?? "") });
      if (typeof value === "string") return new LocalToolResult(value);
      return new LocalToolResult(String(value?.text ?? "(completed)"), value?.display);
    },
    serializeError(error: unknown) {
      return new LocalToolResult("ERROR: " + (error instanceof Error ? error.message : String(error)), undefined, true);
    },
    async render(_ctx: unknown, result: LocalToolResult) {
      return createStringResult(result.text, result.isError);
    },
  };
}

function emptyRequestContext() {
  return new RequestContext({
    env: new RequestContextEnv({
      osVersion: `${process.platform} ${os.release()}`,
      workspacePaths: [process.cwd()],
      shell: process.env.SHELL || "/bin/zsh",
      sandboxEnabled: false,
      terminalsFolder: "",
      agentSharedNotesFolder: "",
      agentConversationNotesFolder: "",
      timeZone: Intl.DateTimeFormat().resolvedOptions().timeZone || "UTC",
      projectFolder: process.cwd(),
      agentTranscriptsFolder: "",
      computerUseSupported: true,
      processWorkingDirectory: process.cwd(),
    }),
    rulesInfoComplete: true,
    envInfoComplete: true,
    repositoryInfoComplete: true,
    customSubagentsInfoComplete: true,
    agentSkillsInfoComplete: true,
    mcpFileSystemInfoComplete: true,
    gitStatusInfoComplete: true,
    mcpInfoComplete: true,
    sendMessageEnabled: true,
    webSearchEnabled: true,
    webFetchEnabled: true,
  });
}

function createResourceAccessor() {
  return {
    get(resource: Loose) {
      if (resource?.symbol === requestContextExecutorResource.symbol) {
        return {
          async execute() {
            return new RequestContextResult({
              result: {
                case: "success",
                value: new RequestContextSuccess({ requestContext: emptyRequestContext() }),
              },
            });
          },
        };
      }
      return undefined;
    },
  } as any;
}

export interface LocalAnysphereRuntimeOptions {
  readonly conversationId: string;
  readonly modelId?: string;
  readonly transport: LocalModelTransport;
  readonly systemPrompt: () => string;
  readonly getTools: () => readonly LocalToolDefinition[];
  readonly executeTool: LocalToolExecutor;
  readonly onUpdate?: (update: Loose) => void;
}

export function createLocalAnysphereRuntime(options: LocalAnysphereRuntimeOptions) {
  const modelId = options.modelId || String(process.env.FABUSHI_AGENT_MODEL || "gpt-5.6");
  const blobStore = new InMemoryBlobStore();
  const resourceAccessor = createResourceAccessor();
  let state = new ConversationStateStructure();
  let activeTools: Loose[] = [];
  let accumulatedText = "";
  let lastUsage: Loose | undefined;

  const getExecutor = (messages?: readonly Loose[]) => new FabushiPromptExecutor(options.transport, messages);
  const toolSession = createTurnToolSession({
    getExecutor: () => getExecutor(),
    isSubagentRunner: false,
    isSilenceAllowed: true,
  });
  const summarizationSession = {
    getModelId: () => modelId,
    getExecutor: (messages: readonly Loose[]) => getExecutor(messages),
  };
  const config = createSandAgentStaticConfig({
    modelId,
    agentTokenLimit: 200_000,
    conversationId: options.conversationId,
    isBoxScopedSubagent: false,
    isSubagentRunner: false,
    isSharedRoomRunner: false,
    sandSendMessageDeliveryOwed: false,
    systemPromptGenerator: () => options.systemPrompt(),
    toolsGenerator: () => ToolSetHandle.fromTools(activeTools),
  });
  const agent = createTurnAgentForRun({
    config,
    toolSession,
    emitUpdate: update => {
      options.onUpdate?.(update);
      if (update?.type === "text-delta" && typeof update.text === "string") accumulatedText += update.text;
      if (update?.type === "turn-ended" && update.usage) lastUsage = update.usage;
    },
    interactionObservers: {},
    privacyMode: PrivacyMode.UNSPECIFIED,
    resourceAccessor,
    blobStore,
    summarizationSession,
  });

  return {
    engine: "grok-anysphere-agent",
    async run(input: { readonly prompt: string; readonly messageId?: string; readonly signal?: AbortSignal; readonly tools?: readonly LocalToolDefinition[] }) {
      accumulatedText = "";
      lastUsage = undefined;
      activeTools = (input.tools ?? options.getTools()).map(definition => makeTool(definition, options.executeTool));
      const action = new ConversationAction({
        action: {
          case: "userMessageAction",
          value: new UserMessageAction({
            userMessage: new UserMessage({
              text: input.prompt,
              messageId: input.messageId ?? crypto.randomUUID(),
            }),
          }),
        },
      });
      const ctx = createContext();
      const [runCtx, cancel] = ctx.withCancel();
      const onAbort = () => cancel(input.signal?.reason);
      if (input.signal?.aborted) onAbort();
      else input.signal?.addEventListener("abort", onAbort, { once: true });
      try {
        const stream = createTurnAgentStreamStart({
          agent: { agent } as any,
          baseState: state,
          action,
          privacyMode: PrivacyMode.UNSPECIFIED,
          mcpTools: [],
        });
        state = await stream.startStream(runCtx, undefined, checkpoint => { state = checkpoint; });
        return { text: accumulatedText, usage: lastUsage, engine: "grok-anysphere-agent" };
      } finally {
        input.signal?.removeEventListener("abort", onAbort);
      }
    },
    snapshot() {
      return { engine: "grok-anysphere-agent", stateBytes: state.toBinary().byteLength, toolCount: activeTools.length };
    },
  };
}
