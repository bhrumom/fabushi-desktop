export function shouldBypassProxy(url: string | URL): boolean {
  try {
    const parsed = typeof url === "string" ? new URL(url) : url;
    const rawHostname = parsed.hostname.toLowerCase();
    const hostname = rawHostname.startsWith("[") && rawHostname.endsWith("]")
      ? rawHostname.slice(1, -1)
      : rawHostname;
    return hostname === "localhost"
      || hostname.endsWith(".localhost")
      || hostname === "::1"
      || hostname === "127.0.0.1";
  } catch {
    return false;
  }
}
