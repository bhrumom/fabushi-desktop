import { ProxyAgent, fetch as undiciFetch } from "undici";
import { shouldBypassProxy } from "./proxy-routing.js";

export { shouldBypassProxy } from "./proxy-routing.js";

const proxyUrl = process.env.HTTPS_PROXY || process.env.https_proxy || undefined;
let proxyAgent: ProxyAgent | undefined;
try { proxyAgent = proxyUrl !== undefined && proxyUrl.length > 0 ? new ProxyAgent(proxyUrl) : undefined; } catch { proxyAgent = undefined; }

export function proxyFetch(url: string | URL, options?: RequestInit): Promise<Response> {
  const useProxy = proxyAgent !== undefined && !shouldBypassProxy(url);
  const undiciOptions = (useProxy ? { ...options, dispatcher: proxyAgent } : options) as Parameters<typeof undiciFetch>[1];
  return undiciFetch(url, undiciOptions) as unknown as Promise<Response>;
}
