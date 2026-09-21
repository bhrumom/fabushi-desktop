import React, { useMemo } from 'react';
import type { MahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import { FabButton, FabSurface } from '../../ui/primitives/fab-primitives';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import { useCompatibilityMessagingRuntime } from '../compatibility/use-compatibility-messaging-runtime';
import styles from '../compatibility/compatibility-data.module.css';

export default function CallsCompatibilityAdapter({
  transport,
  onClose,
}: {
  readonly transport: MahayanaHostTransport;
  readonly onClose: () => void;
}) {
  const runtime = useCompatibilityMessagingRuntime(transport);
  const callCapable = useMemo(
    () => runtime.conversations.filter((conversation) => conversation.permissions.canManageCalls),
    [runtime.conversations],
  );
  return <CompatibilityFeatureFrame
    title="Calls"
    description="Voice/video signaling remains a messaging capability and is not projected as Agent lifecycle state."
    onClose={onClose}
  >
    <div className={styles.toolbar}>
      <span className={styles.status}>{callCapable.length} call-capable conversations</span>
      <FabButton type="button" variant="ghost" onClick={() => void runtime.refresh()}>Refresh</FabButton>
    </div>
    <div className={styles.list}>
      {callCapable.map((conversation) => <FabSurface className={styles.row} key={conversation.id}>
        <span className={styles.copy}><strong>{conversation.title}</strong><small>{conversation.kind} · WebRTC compatibility</small></span>
        <span />
        <span className={styles.meta}>Available</span>
      </FabSurface>)}
      {!runtime.loading && !callCapable.length ? <div className={styles.status}>No call-capable conversations are available.</div> : null}
    </div>
    {runtime.error ? <div className={styles.error} role="alert">{runtime.error}</div> : null}
  </CompatibilityFeatureFrame>;
}
