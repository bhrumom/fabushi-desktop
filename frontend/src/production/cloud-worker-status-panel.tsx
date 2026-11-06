import { useEffect, useState, type ReactElement } from "react";
import { parseCloudWorkerCapabilitiesProjection, parseCloudWorkerStatusEvent, parseCloudWorkerStatusProjection, type CloudWorkerCapabilitiesProjection, type CloudWorkerStatusProjection } from "../../../source/shared/rpc/coordinator";
import type { ProductionCoordinatorClient } from "./coordinator-client";

type CloudWorkerStatusState =
  | { readonly kind: "loading" }
  | { readonly kind: "ready"; readonly projection: CloudWorkerStatusProjection; readonly capabilities: CloudWorkerCapabilitiesProjection }
  | { readonly kind: "error"; readonly message: string };

export function CloudWorkerStatusPanel({ client }: { readonly client: ProductionCoordinatorClient | null }): ReactElement {
  const [state, setState] = useState<CloudWorkerStatusState>({ kind: "loading" });

  useEffect(() => {
    if (client == null) {
      setState({ kind: "error", message: "Coordinator is unavailable." });
      return;
    }
    let active = true;
    const applyProjections = ([statusValue, capabilitiesValue]: readonly [unknown, unknown]) => {
      const projection = parseCloudWorkerStatusProjection(statusValue);
      const capabilities = parseCloudWorkerCapabilitiesProjection(capabilitiesValue);
      if (active && projection != null && capabilities != null) setState({ kind: "ready", projection, capabilities });
    };
    const applyEvent = (value: unknown) => {
      const eventProjection = parseCloudWorkerStatusEvent(value);
      if (active && eventProjection != null) {
        setState((current) => current.kind === "ready" ? { ...current, projection: eventProjection } : current);
      }
    };
    void Promise.all([client.cloudWorkersStatus(), client.cloudWorkersCapabilities()]).then(applyProjections).catch((reason: unknown) => {
      if (active) setState({ kind: "error", message: reason instanceof Error ? reason.message : "Cloud worker status is unavailable." });
    });
    const unsubscribe = client.subscribe("cloud-workers.status", applyEvent);
    return () => {
      active = false;
      unsubscribe();
    };
  }, [client]);

  if (state.kind === "loading") return <div aria-live="polite" role="status">Loading cloud worker status…</div>;
  if (state.kind === "error") return <div aria-live="polite" role="alert">{state.message}</div>;
  const { projection } = state;
  return (
    <section aria-label="Cloud worker status">
      <h3>Cloud workers</h3>
      <p>Read-only status from the canonical Host owner.</p>
      <dl>
        <dt>Status</dt><dd>{projection.status}</dd>
        <dt>Provider readiness</dt><dd>{projection.providerReady ? "Ready" : "Not ready"}</dd>
        <dt>Production mutation</dt><dd>{projection.productionMutation ? "Enabled" : "Not enabled"}</dd>
        <dt>State persisted</dt><dd>{projection.statePersisted ? "Yes" : "No"}</dd>
        <dt>Leases</dt><dd>{projection.leaseCount}</dd>
        <dt>Warm images</dt><dd>{projection.warmImageCount}</dd>
        <dt>Bootstraps</dt><dd>{projection.bootstrapCount}</dd>
        <dt>Artifacts</dt><dd>{projection.artifactCount}</dd>
        <dt>Migration receipts</dt><dd>{projection.migrationReceiptCount}</dd>
      </dl>
      <h4>Available provider metadata</h4>
      {state.capabilities.providers.length > 0 ? (
        <ul>
          {state.capabilities.providers.map((provider) => <li key={`${provider.pluginId}:${provider.providerId}`}>{provider.label ?? provider.providerId} ({provider.pluginId})</li>)}
        </ul>
      ) : <p>No worker provider metadata is registered.</p>}
    </section>
  );
}
