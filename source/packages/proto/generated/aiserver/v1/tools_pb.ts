/** Renderer-only type closure for the reconstructed ClientSideToolV2 cards.
 * Runtime tool execution remains owned by Fabushi's local agent host.
 */
export type ClientSideToolV2 = number;
export interface EditFileParams { readonly relativeWorkspacePath: string; }
export interface EditFileV2Params { readonly relativeWorkspacePath: string; readonly isStreaming?: boolean; }
export interface RunTerminalCommandV2Params { readonly command: string; readonly cwd?: string; readonly isBackground: boolean; }
export interface ToolResultError { readonly clientVisibleErrorMessage: string; }
export interface DiffLike { readonly chunks: readonly { readonly diffString: string }[]; }
export interface EditFileResult { readonly rejected?: boolean; readonly applyFailed?: boolean; readonly recoverableError?: unknown; readonly isApplied: boolean; readonly diff?: DiffLike; }
export interface EditFileV2Result { readonly rejected?: boolean; readonly fileWasCreated: boolean; readonly diff?: DiffLike; }
export interface RunTerminalCommandV2Result { readonly outputRaw: string; readonly output: string; readonly rejected?: boolean; readonly poppedOutIntoBackground: boolean; readonly isRunningInBackground: boolean; readonly endedReason: number; }
export interface ClientSideToolV2Call {
  readonly toolCallId: string;
  readonly tool: ClientSideToolV2;
  readonly params:
    | { readonly case: "editFileParams"; readonly value: EditFileParams }
    | { readonly case: "editFileV2Params"; readonly value: EditFileV2Params }
    | { readonly case: "runTerminalCommandV2Params"; readonly value: RunTerminalCommandV2Params }
    | { readonly case: string | undefined; readonly value?: unknown };
}
export interface ClientSideToolV2Result {
  readonly toolCallId: string;
  readonly tool: ClientSideToolV2;
  readonly error?: ToolResultError;
  readonly result:
    | { readonly case: "editFileResult"; readonly value: EditFileResult }
    | { readonly case: "editFileV2Result"; readonly value: EditFileV2Result }
    | { readonly case: "runTerminalCommandV2Result"; readonly value: RunTerminalCommandV2Result }
    | { readonly case: string | undefined; readonly value?: unknown };
}
