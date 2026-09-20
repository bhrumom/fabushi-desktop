import { useCallback, useEffect, useState } from 'react';

export interface AgentCommandPaletteController {
  readonly open: boolean;
  readonly query: string;
  setQuery(value: string): void;
  openPalette(): void;
  close(): void;
  toggle(): void;
}

/**
 * Owns the global Agent command surface lifecycle and keyboard contract.
 *
 * Cmd/Ctrl+K is an Agent-shell concern, not Messenger navigation state.
 */
export function useAgentCommandPaletteController(): AgentCommandPaletteController {
  const [open, setOpen] = useState(false);
  const [query, setQuery] = useState('');

  const openPalette = useCallback(() => {
    setQuery('');
    setOpen(true);
  }, []);

  const close = useCallback(() => setOpen(false), []);

  const toggle = useCallback(() => {
    setQuery('');
    setOpen((value) => !value);
  }, []);

  useEffect(() => {
    const handleKeys = (event: KeyboardEvent) => {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        toggle();
        return;
      }
      if (event.key === 'Escape') close();
    };
    window.addEventListener('keydown', handleKeys);
    return () => window.removeEventListener('keydown', handleKeys);
  }, [close, toggle]);

  return { open, query, setQuery, openPalette, close, toggle };
}
