export function formatTranscriptToolCallName(name: string): string {
  const withoutSuffix = name.endsWith("ToolCall") ? name.slice(0, -8) : name;
  if (withoutSuffix.length === 0) return name;
  const spaced = withoutSuffix.replace(/([a-z0-9])([A-Z])/g, "$1 $2");
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}
