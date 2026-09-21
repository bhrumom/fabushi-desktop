import React, { useMemo, useState } from 'react';
import type { MahayanaHostTransport } from '../../../../frontend/apps/web/src/lib/mahayana-host/transport';
import FabAvatar from '../../ui/avatar/fab-avatar';
import { FabButton, FabInput, FabSurface } from '../../ui/primitives/fab-primitives';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import { useCompatibilityMessagingRuntime } from '../compatibility/use-compatibility-messaging-runtime';
import styles from '../compatibility/compatibility-data.module.css';

export default function ContactsCompatibilityAdapter({
  transport,
  onClose,
}: {
  readonly transport: MahayanaHostTransport;
  readonly onClose(): void;
}) {
  const runtime = useCompatibilityMessagingRuntime(transport);
  const [query, setQuery] = useState('');
  const contacts = useMemo(() => {
    const normalized = query.trim().toLowerCase();
    return runtime.actors
      .filter((actor) => actor.kind === 'human' && actor.id !== runtime.client.actorId)
      .filter((actor) => !normalized || `${actor.displayName} ${actor.username ?? ''} ${actor.bio ?? ''}`.toLowerCase().includes(normalized))
      .sort((left, right) => Number(right.presence.status === 'online') - Number(left.presence.status === 'online')
        || left.displayName.localeCompare(right.displayName));
  }, [query, runtime.actors, runtime.client.actorId]);

  return <CompatibilityFeatureFrame
    title="Contacts"
    description="Human identities are projected from the messaging core and remain separate from Agent identities."
    onClose={onClose}
  >
    <div className={styles.toolbar}>
      <FabInput value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search contacts" aria-label="Search contacts" />
      <FabButton type="button" variant="ghost" onClick={() => void runtime.refresh()}>Refresh</FabButton>
    </div>
    {runtime.loading ? <div className={styles.status}>Loading contacts…</div> : null}
    {runtime.error ? <div className={styles.error} role="alert">{runtime.error}</div> : null}
    <div className={styles.list}>
      {contacts.map((actor) => <FabSurface className={styles.row} key={actor.id}>
        <FabAvatar identity={`contact:${actor.id}`} state={actor.presence.status === 'online' ? 'idle' : 'offline'} size={36} label={actor.displayName} />
        <span className={styles.copy}>
          <strong>{actor.displayName}</strong>
          <small>{actor.username ? `@${actor.username}` : actor.bio || 'Fabushi contact'}</small>
        </span>
        <span className={styles.meta}>{actor.presence.status}</span>
      </FabSurface>)}
      {!runtime.loading && !contacts.length ? <div className={styles.status}>No matching contacts.</div> : null}
    </div>
  </CompatibilityFeatureFrame>;
}
