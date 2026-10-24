import type { JsonSchema, McpTool } from "./types";

export type ParsedToolCommand =
  | { kind: "call"; tool: McpTool; arguments: Record<string, unknown> }
  | { kind: "form"; tool: McpTool; initial: Record<string, unknown> }
  | { kind: "text"; text: string };

export function parseToolCommand(input: string, tools: McpTool[]): ParsedToolCommand {
  if (!input.startsWith("/")) return { kind: "text", text: input };
  const separator = input.search(/\s/);
  const name = input.slice(1, separator < 0 ? undefined : separator);
  const remainder = separator < 0 ? "" : input.slice(separator + 1);
  const tool = tools.find((candidate) => candidate.name === name);
  if (!tool) throw new Error(`当前插件没有 /${name} Tool`);
  const properties = tool.inputSchema?.properties ?? {};
  const fields = Object.entries(properties);
  if (fields.length === 0) return { kind: "call", tool, arguments: {} };
  if (fields.length === 1 && fields[0][1].type === "string") {
    return { kind: "call", tool, arguments: { [fields[0][0]]: remainder } };
  }
  return { kind: "form", tool, initial: schemaDefaults(tool.inputSchema) as Record<string, unknown> };
}

export function schemaDefaults(schema?: JsonSchema): unknown {
  if (!schema) return undefined;
  if (schema.default !== undefined) return schema.default;
  if (schema.type === "object") {
    return Object.fromEntries(
      Object.entries(schema.properties ?? {})
        .map(([key, child]) => [key, schemaDefaults(child)])
        .filter(([, value]) => value !== undefined),
    );
  }
  if (schema.type === "array") return [];
  if (schema.type === "boolean") return false;
  return undefined;
}

export function validateSchemaValue(
  schema: JsonSchema | undefined,
  value: unknown,
  path = "$",
): string[] {
  if (!schema) return [];
  const errors: string[] = [];
  if (schema.enum && !schema.enum.some((candidate) => Object.is(candidate, value))) {
    errors.push(`${path} 必须是枚举中的一个值`);
    return errors;
  }
  if (schema.type === "object" || schema.properties) {
    if (!value || typeof value !== "object" || Array.isArray(value)) {
      return [`${path} 必须是对象`];
    }
    const object = value as Record<string, unknown>;
    for (const required of schema.required ?? []) {
      if (object[required] === undefined || object[required] === null || object[required] === "") {
        errors.push(`${path}.${required} 是必填项`);
      }
    }
    for (const [key, child] of Object.entries(schema.properties ?? {})) {
      if (object[key] !== undefined) errors.push(...validateSchemaValue(child, object[key], `${path}.${key}`));
    }
  } else if (schema.type === "array") {
    if (!Array.isArray(value)) return [`${path} 必须是数组`];
    value.forEach((item, index) => errors.push(...validateSchemaValue(schema.items, item, `${path}[${index}]`)));
  } else if (schema.type === "string" && typeof value !== "string") {
    errors.push(`${path} 必须是字符串`);
  } else if (schema.type === "boolean" && typeof value !== "boolean") {
    errors.push(`${path} 必须是布尔值`);
  } else if (schema.type === "number" && typeof value !== "number") {
    errors.push(`${path} 必须是数字`);
  } else if (schema.type === "integer" && !Number.isInteger(value)) {
    errors.push(`${path} 必须是整数`);
  }
  return errors;
}
