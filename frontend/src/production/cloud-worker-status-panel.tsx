import { useEffect, useState, type ReactElement } from "react";
import { parseCloudWorkerStatusEvent, parseCloudWorkerStatusProjection, type CloudWorkerStatusProjection } from "../../../source/shared/rpc/coordinator";
import type { ProductionCoordinatorClient } from "./coordinator-client";

type CloudWorkerStatusState =
  | { readonly kind: "loading" }
  | { readonly kind: "ready"; readonly projection: CloudWorkerStatusProjection }
  | { readonly kind: "error"; readonly message: string };

export function CloudWorkerStatusPanel({ client }: { readonly client: ProductionCoordinatorClient | null }): ReactElement {
  const [state, setState] = useState<CloudWorkerStatusState>({ kind: "loading" });

  useEffect(() => {
    if (client == null) {
      setState({ kind: "error", message: "Coordinator is unavailable." });
      return;
    }
    let active = true;
    const applyProjection = (value: unknown) => {
      const projection = parseCloudWorkerStatusProjection(value);
      if (active && projection != null) setState({ kind: "ready", projection });
    };
    const applyEvent = (value: unknown) => {
      const projection = parseCloudWorkerStatusEvent(value);
      if (active && projection != null) setState({ kind: "ready", projection });
    };
    void client.cloudWorkersStatus().then(applyProjection).catch((reason: unknown) => {
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
    </section>
  );
}
