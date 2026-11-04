/**
 * Sandboxed preload for Fabushi Mini App windows.
 *
 * Exposes exactly one function. It cannot read files, spawn processes or reach
 * the network: every call is a request the Main process may refuse.
 */
import { contextBridge, ipcRenderer } from "electron";
import { MINIAPP_TOOL_CHANNEL } from "../shared/miniapp-tool-contract.js";

export { MINIAPP_TOOL_CHANNEL } from "../shared/miniapp-tool-contract.js";

export interface MiniAppLaunchContext {
  readonly launchSessionId?: string;
  readonly conversationId?: string;
  readonly startParameter?: string;
}

export interface MiniAppConversationBridge {
  openInvoice(invoiceId: string): Promise<unknown>;
  startServiceCall(callId: string, mode: "voice" | "text" | "hybrid"): Promise<unknown>;
  submitServiceCallInput(callId: string, input: Record<string, unknown>): Promise<unknown>;
  endServiceCall(callId: string): Promise<unknown>;
  sendMessage?(text: string): Promise<unknown>;
  sendData?(data: string): Promise<unknown>;
}

export interface MiniAppToolBridge {
  invoke(tool: string, args?: Record<string, unknown>): Promise<unknown>;
  getLaunchContext(): Promise<MiniAppLaunchContext>;
  conversation: MiniAppConversationBridge;
  openInvoice(invoiceId: string): Promise<unknown>;
  startServiceCall(callId: string, mode: "voice" | "text" | "hybrid"): Promise<unknown>;
  submitServiceCallInput(callId: string, input: Record<string, unknown>): Promise<unknown>;
  endServiceCall(callId: string): Promise<unknown>;
}

const conversationBridge: MiniAppConversationBridge = Object.freeze({
  openInvoice: (invoiceId: string) => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, "conversation.openInvoice", { invoiceId }),
  startServiceCall: (callId: string, mode: "voice" | "text" | "hybrid") => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, "conversation.startServiceCall", { callId, mode }),
  submitServiceCallInput: (callId: string, input: Record<string, unknown>) => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, "conversation.submitServiceCallInput", { callId, input }),
  endServiceCall: (callId: string) => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, "conversation.endServiceCall", { callId }),
  sendMessage: (text: string) => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, "conversation.sendMessage", { text }),
  sendData: (data: string) => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, "conversation.sendData", { data }),
});

const bridge: MiniAppToolBridge = Object.freeze({
  invoke: (tool: string, args?: Record<string, unknown>) => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, tool, args ?? {}),
  getLaunchContext: async (): Promise<MiniAppLaunchContext> => {
    const res = (await ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, "launch.getContext", {})) as
      | { readonly ok: true; readonly data?: MiniAppLaunchContext }
      | { readonly ok: false }
      | undefined;
    return res && res.ok && res.data ? res.data : {};
  },
  conversation: conversationBridge,
  openInvoice: (invoiceId: string) => conversationBridge.openInvoice(invoiceId),
  startServiceCall: (callId: string, mode: "voice" | "text" | "hybrid") => conversationBridge.startServiceCall(callId, mode),
  submitServiceCallInput: (callId: string, input: Record<string, unknown>) => conversationBridge.submitServiceCallInput(callId, input),
  endServiceCall: (callId: string) => conversationBridge.endServiceCall(callId),
});

contextBridge.exposeInMainWorld("fabushi", bridge);
