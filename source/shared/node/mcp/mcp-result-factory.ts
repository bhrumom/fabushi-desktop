import {
  McpError,
  McpRejected,
  McpResult,
  McpSuccess,
  McpTextContent,
  McpToolResultContentItem,
} from "../../../packages/proto/generated/agent/v1/mcp_exec_pb.js";
import type {
  McpContentItem,
  McpResultFactory,
  McpResultLike,
} from "./mcp-image-assets.js";

export const MCP_ORIGIN_RETIRED = "MCP_ORIGIN_RETIRED";

/** Only native owner assertions create this known pre-dispatch refusal. */
export class McpOriginRetiredError extends Error {
  constructor() {
    super(MCP_ORIGIN_RETIRED);
    this.name = "McpOriginRetiredError";
  }
}

export type McpDispatchAssertion = () => undefined;

/** Exact constructor projection used by the shipped shared MCP runtime. */
export const generatedMcpResultFactory = {
  textItem(text: string): McpToolResultContentItem {
    return new McpToolResultContentItem({
      content: {
        case: "text",
        value: new McpTextContent({ text }),
      },
    });
  },

  success<T extends McpResultLike>(original: T, content: McpContentItem[]): T {
    const current = original.result.value as {
      isError?: boolean;
      structuredContent?: McpSuccess["structuredContent"];
    };
    return new McpResult({
      result: {
        case: "success",
        value: new McpSuccess({
          content: content as McpToolResultContentItem[],
          isError: current.isError ?? false,
          ...(current.structuredContent === undefined
            ? {}
            : { structuredContent: current.structuredContent }),
        }),
      },
    }) as unknown as T;
  },

  error(message: string): McpResult {
    return new McpResult({
      result: {
        case: "error",
        value: new McpError({ error: message }),
      },
    });
  },

  rejected(reason: string): McpResult {
    return new McpResult({
      result: { case: "rejected", value: new McpRejected({ reason }) },
    });
  },
} satisfies McpResultFactory & {
  error(message: string): McpResult;
  rejected(reason: string): McpResult;
};
