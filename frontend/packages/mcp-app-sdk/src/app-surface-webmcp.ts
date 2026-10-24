import {
  FABUSHI_APP_TOOL_NAMES,
  appSurfaceToolDefinitions,
  type AppSurface,
  type AppSurfaceOperation,
} from "./app-surface";
import { registerWebMcpTool } from "./webmcp";

const operationByName = new Map<string, AppSurfaceOperation>(
  Object.entries(FABUSHI_APP_TOOL_NAMES)
    .map(([operation, name]) => [name, operation as AppSurfaceOperation]),
);

export function registerAppSurfaceWebMcp(surface: AppSurface): () => void {
  const disposers = appSurfaceToolDefinitions().map((definition) => {
    const operation = operationByName.get(definition.name);
    if (!operation) throw new Error(`Unknown Fabushi App Surface tool: ${definition.name}`);
    return registerWebMcpTool({
      name: definition.name,
      title: definition.title,
      description: definition.description,
      inputSchema: definition.inputSchema,
      annotations: { readOnlyHint: definition.annotations.readOnlyHint },
      execute: (input, options) => surface.call(operation, input, options),
    });
  });
  return () => {
    for (const dispose of disposers.reverse()) dispose();
  };
}
