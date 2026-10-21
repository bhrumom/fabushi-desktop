import { ProxyAgent, fetch as undiciFetch } from "undici";

const proxyUrl = process.env.HTTPS_PROXY || process.env.https_proxy || undefined;
let proxyAgent: ProxyAgent | undefined;
try { proxyAgent = proxyUrl !== undefined && proxyUrl.length > 0 ? new ProxyAgent(proxyUrl) : undefined; } catch { proxyAgent = undefined; }

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

export function proxyFetch(url: string | URL, options?: RequestInit): Promise<Response> {
  const useProxy = proxyAgent !== undefined && !shouldBypassProxy(url);
  const undiciOptions = (useProxy ? { ...options, dispatcher: proxyAgent } : options) as Parameters<typeof undiciFetch>[1];
  return undiciFetch(url, undiciOptions) as unknown as Promise<Response>;
}
