import { Search, X } from 'lucide-react';
import React, { useEffect, useMemo, useRef } from 'react';
import { FabButton, FabInput } from '../ui/primitives/fab-primitives';
import type { TranscriptEntry } from './transcript-model';
import styles from './agent-search.module.css';

export interface AgentSearchProps {
  entries: readonly TranscriptEntry[];
  query: string;
  onQuery(value: string): void;
  onClose(): void;
  onSelect(entryId: string): void;
}

export default function AgentSearch({ entries, query, onQuery, onClose, onSelect }: AgentSearchProps) {
  const inputRef = useRef<HTMLInputElement | null>(null);
  useEffect(() => { inputRef.current?.focus(); }, []);

  const normalized = query.trim().toLocaleLowerCase();
  const results = useMemo(() => {
    if (!normalized) return entries.slice(-12).reverse();
    return entries.filter((entry) => {
      const haystack = [entry.text, entry.title, entry.detail]
        .filter(Boolean)
        .join(' ')
        .toLocaleLowerCase();
      return haystack.includes(normalized);
    }).slice(-60).reverse();
  }, [entries, normalized]);

  return <section className={styles.root} data-testid="agent-search" aria-label="Search this Agent">
    <label>
      <Search size={15} />
      <FabInput variant="bare"
        ref={inputRef}
        value={query}
        onChange={(event) => onQuery(event.target.value)}
        placeholder="Search this Agent"
        aria-label="Search this Agent"
      />
      <FabButton variant="bare" type="button" onClick={onClose} aria-label="Close search"><X size={14} /></FabButton>
    </label>
    <div className={styles.results}>
      {results.length ? results.map((entry) => <FabButton variant="bare"
        type="button"
        key={entry.id}
        onClick={() => onSelect(entry.id)}
        data-entry-kind={entry.kind}
      >
        <span><strong>{entry.role === 'me' ? 'You' : entry.title || 'Agent'}</strong><small>{entry.kind}</small></span>
        <p>{entry.text || entry.detail || entry.title || 'Timeline event'}</p>
      </FabButton>) : <p className={styles.empty}>No matching messages in the loaded transcript.</p>}
    </div>
  </section>;
}
