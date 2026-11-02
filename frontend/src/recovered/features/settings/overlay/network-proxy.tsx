import { useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";
import type { ProductNetworkProxyEndpoint, ProductNetworkProxyMutation, ProductNetworkProxySettings, ProductNetworkProxyType } from "../../../contracts/desktop-bridge";
import { SandButton } from "../../../ui/sand-kit-primitives";
import { SandSelect } from "../../../ui/sand-floating-primitives";
import { SandSwitch } from "../../../ui/sand-form-primitives";

export function preferredIndicesWithToggle(indices: readonly number[], index: number, enabled: boolean): number[] {
  const without = indices.filter((value) => value !== index);
  return enabled ? [...without, index] : without;
}

const TYPE_OPTIONS = [{ value: "socks5", label: "SOCKS5" }, { value: "http", label: "HTTP" }, { value: "mtproto", label: "MTProto" }, { value: "web", label: "Web proxy" }] as const;
const MODE_OPTIONS = [{ value: "system", label: "System" }, { value: "enabled", label: "Selected proxy" }, { value: "disabled", label: "Direct" }] as const;
const EMPTY_PROXY: ProductNetworkProxyEndpoint = { type: "socks5", host: "", port: 1080, user: "", password: "" };
const sameProxy = (left: ProductNetworkProxyEndpoint | null, right: ProductNetworkProxyEndpoint) => left != null && JSON.stringify(left) === JSON.stringify(right);
type MutateProxyPreference = (mutation: ProductNetworkProxyMutation) => Promise<ProductNetworkProxySettings | null>;

function ProxyEndpointEditor({ proxy, index, selected, preferred, pending, onMutate }: {
  proxy: ProductNetworkProxyEndpoint;
  index: number;
  selected: boolean;
  preferred: boolean;
  pending: boolean;
  onMutate: MutateProxyPreference;
}) {
  const [draft, setDraft] = useState(proxy);
  const committed = useRef(proxy);
  const lifetime = useRef(0);
  useLayoutEffect(() => () => { lifetime.current += 1; }, []);
  useEffect(() => {
    // An unrelated preference save can return newly allocated endpoint
    // objects. Do not erase a local edit when their committed value is equal.
    if (!sameProxy(committed.current, proxy)) setDraft(proxy);
    committed.current = proxy;
  }, [proxy]);

  const save = async () => {
    if (pending) return;
    const generation = lifetime.current;
    const result = await onMutate({
      type: "replace",
      was: proxy,
      proxy: { ...draft, port: Math.max(0, Math.trunc(Number(draft.port) || 0)) },
    });
    if (result != null && generation === lifetime.current && result.list[index] != null) {
      setDraft(result.list[index]!);
    }
  };

  return <div className="sand-network-proxy-endpoint">
    <div className="sand-network-proxy-endpoint__header"><strong>Proxy {index + 1}</strong><span>{selected ? "Selected" : preferred ? "Rotation preferred" : ""}</span></div>
    <div className="sand-network-proxy-endpoint__fields">
      <label><span>Type</span><SandSelect ariaLabel={"Proxy " + (index + 1) + " type"} className="ui-select-trigger" disabled={pending} onValueChange={(type) => { if (!pending) setDraft((current) => ({ ...current, type: type as ProductNetworkProxyType })); }} options={TYPE_OPTIONS.map((option) => ({ ...option, disabled: pending }))} placement="bottom-end" value={draft.type} /></label>
      <label><span>Host</span><input aria-label={"Proxy " + (index + 1) + " host"} disabled={pending} onChange={(event) => { const host = event.currentTarget.value; setDraft((current) => ({ ...current, host })); }} value={draft.host} /></label>
      <label><span>Port</span><input aria-label={"Proxy " + (index + 1) + " port"} disabled={pending} inputMode="numeric" min={0} onChange={(event) => { const port = Number(event.currentTarget.value); setDraft((current) => ({ ...current, port })); }} type="number" value={draft.port} /></label>
      <label><span>User</span><input aria-label={"Proxy " + (index + 1) + " user"} autoComplete="off" disabled={pending} onChange={(event) => { const user = event.currentTarget.value; setDraft((current) => ({ ...current, user })); }} value={draft.user} /></label>
      <label><span>Password / secret</span><input aria-label={"Proxy " + (index + 1) + " password or secret"} autoComplete="new-password" disabled={pending} onChange={(event) => { const password = event.currentTarget.value; setDraft((current) => ({ ...current, password })); }} type="password" value={draft.password} /></label>
    </div>
    <div className="sand-network-proxy-endpoint__actions">
      <SandButton disabled={pending} onClick={() => { void save(); }} size="sm" variant="secondary">Save</SandButton>
      <SandButton disabled={pending} onClick={() => { void onMutate({ type: "remove", proxy }); }} size="sm" variant="secondary">Remove</SandButton>
    </div>
  </div>;
}

export interface NetworkProxySettingsPanelProps {
  state: ProductNetworkProxySettings;
  pending: boolean;
  availability: "unavailable" | "unconfirmed";
  onMutate: MutateProxyPreference;
}

export function NetworkProxySettingsPanel({ state, pending, availability, onMutate }: NetworkProxySettingsPanelProps) {
  const selectedIndex = state.list.findIndex((proxy) => sameProxy(state.selected, proxy));
  const selectedOptions = useMemo(() => [{ value: "none", label: "No selected proxy", disabled: pending }, ...state.list.map((proxy, index) => ({ value: String(index), label: proxy.host.length > 0 ? (index + 1) + ". " + proxy.host + ":" + proxy.port : (index + 1) + ". Unnamed proxy", disabled: pending }))], [state.list, pending]);
  const [timeoutDraft, setTimeoutDraft] = useState(String(state.rotation.timeoutSeconds));
  const inFlight = useRef<symbol | null>(null);
  const lifetime = useRef(0);
  useLayoutEffect(() => () => { lifetime.current += 1; }, []);
  useEffect(() => { setTimeoutDraft(String(state.rotation.timeoutSeconds)); }, [state.rotation.timeoutSeconds]);

  const mutate: MutateProxyPreference = async (mutation) => {
    // The surface owns dispatch/lifetime policy. This synchronous guard also
    // prevents another control from dispatching before pending is rendered.
    if (pending || inFlight.current != null) return null;
    const request = Symbol();
    const generation = lifetime.current;
    inFlight.current = request;
    try {
      const result = await onMutate(mutation);
      return generation === lifetime.current ? result : null;
    } catch {
      // The Settings surface presents a fixed failure notice. An unexpected
      // rejection must not become an unhandled event-handler promise.
      return null;
    } finally {
      if (inFlight.current === request) inFlight.current = null;
    }
  };

  const saveTimeout = async () => {
    if (timeoutDraft === String(state.rotation.timeoutSeconds)) return;
    const result = await mutate({ type: "setRotationTimeout", seconds: Number(timeoutDraft) });
    // A successful 10 -> 0 -> 10 normalization does not change the state prop.
    // Consume the returned value explicitly rather than waiting for an effect.
    if (result != null) setTimeoutDraft(String(result.rotation.timeoutSeconds));
  };

  return <fieldset aria-busy={pending} aria-label="Network proxy preferences" className="sand-network-proxy-settings" disabled={pending} style={{ border: 0, margin: 0, padding: 0, minWidth: 0 }}>
    <p className="sand-settings-a11y-status" role="status" aria-live="polite">{availability === "unavailable"
      ? "Proxy connections are unavailable. Your preferences can still be saved."
      : "Proxy connection availability has not been confirmed."}</p>
    {pending ? <p className="sand-settings-a11y-status" role="status">Saving proxy preferences…</p> : null}
    <label className="sand-settings-row"><span className="sand-settings-copy"><strong>Connection preference</strong><small>Save how you want proxy connections configured.</small></span><SandSelect ariaLabel="Network proxy mode" className="ui-select-trigger" disabled={pending} onValueChange={(mode) => { void mutate({ type: "setMode", mode: mode as ProductNetworkProxySettings["mode"] }); }} options={MODE_OPTIONS.map((option) => ({ ...option, disabled: pending }))} placement="bottom-end" value={state.mode} /></label>
    <label className="sand-settings-row"><span className="sand-settings-copy"><strong>Selected proxy</strong><small>Choose an endpoint from your saved proxy list.</small></span><SandSelect ariaLabel="Selected network proxy" className="ui-select-trigger" disabled={pending} onValueChange={(value) => { void mutate({ type: "setSelected", proxy: value === "none" ? null : state.list[Number(value)] ?? null }); }} options={selectedOptions} placement="bottom-end" value={selectedIndex < 0 ? "none" : String(selectedIndex)} /></label>
    {state.selected != null && selectedIndex < 0 ? <p className="sand-settings-a11y-status" role="status">The saved selection is not in this list.</p> : null}
    <div className="sand-network-proxy-toggles">
      <SandSwitch checked={state.tryIpv6} disabled={pending} label={<span className="sand-settings-copy"><strong>Try IPv6</strong><small>Save a preference for IPv6 when it is supported.</small></span>} onCheckedChange={(enabled) => { void mutate({ type: "setTryIpv6", enabled }); }} />
      <SandSwitch checked={state.useProxyForCalls} disabled={pending} label={<span className="sand-settings-copy"><strong>Use proxy for calls</strong><small>Save your preferred proxy setting for calls.</small></span>} onCheckedChange={(enabled) => { void mutate({ type: "setUseProxyForCalls", enabled }); }} />
      <SandSwitch checked={state.checkIpWarningShown} disabled={pending} label={<span className="sand-settings-copy"><strong>IP warning acknowledged</strong><small>Remember that you have seen the IP warning.</small></span>} onCheckedChange={(shown) => { void mutate({ type: "setCheckIpWarningShown", shown }); }} />
      <SandSwitch checked={state.rotation.enabled} disabled={pending} label={<span className="sand-settings-copy"><strong>Proxy rotation</strong><small>Save the preferred rotation setting.</small></span>} onCheckedChange={(enabled) => { void mutate({ type: "setRotationEnabled", enabled }); }} />
    </div>
    <label className="sand-settings-row"><span className="sand-settings-copy"><strong>Rotation timeout</strong><small>Zero and negative values are saved as 10 seconds.</small></span><input aria-label="Proxy rotation timeout seconds" disabled={pending} min={0} onBlur={() => { void saveTimeout(); }} onChange={(event) => setTimeoutDraft(event.currentTarget.value)} type="number" value={timeoutDraft} /></label>
    <div className="sand-network-proxy-list" aria-label="Configured proxies">
      {state.list.map((proxy, index) => <div key={proxy.type + ":" + proxy.host + ":" + proxy.port + ":" + index}>
        <ProxyEndpointEditor index={index} onMutate={mutate} pending={pending} preferred={state.rotation.preferredIndices.includes(index)} proxy={proxy} selected={index === selectedIndex} />
        <div className="sand-network-proxy-preferred"><SandSwitch checked={state.rotation.preferredIndices.includes(index)} disabled={pending} label={<span className="sand-settings-copy"><strong>Prefer in rotation</strong><small>Keep this proxy in the ordered preferred set.</small></span>} onCheckedChange={(enabled) => { void mutate({ type: "setRotationPreferredIndices", indices: preferredIndicesWithToggle(state.rotation.preferredIndices, index, enabled) }); }} /></div>
      </div>)}
      {state.list.length === 0 ? <p className="sand-settings-a11y-status">No proxies configured.</p> : null}
    </div>
    <SandButton disabled={pending} onClick={() => { void mutate({ type: "add", proxy: { ...EMPTY_PROXY } }); }} size="sm" variant="secondary">Add proxy</SandButton>
  </fieldset>;
}
