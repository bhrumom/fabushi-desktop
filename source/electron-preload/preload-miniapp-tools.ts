/**
 * Sandboxed preload for Fabushi Mini App windows.
 *
 * Exposes exactly one function. It cannot read files, spawn processes or reach
 * the network: every call is a request the Main process may refuse.
 */
import { contextBridge, ipcRenderer } from "electron";
import { MINIAPP_TOOL_CHANNEL } from "../shared/miniapp-tool-contract.js";

export { MINIAPP_TOOL_CHANNEL } from "../shared/miniapp-tool-contract.js";

export interface MiniAppToolBridge {
  invoke(tool: string, args?: Record<string, unknown>): Promise<unknown>;
}

const bridge: MiniAppToolBridge = Object.freeze({
  invoke: (tool: string, args?: Record<string, unknown>) => ipcRenderer.invoke(MINIAPP_TOOL_CHANNEL, tool, args ?? {}),
});

contextBridge.exposeInMainWorld("fabushi", bridge);
