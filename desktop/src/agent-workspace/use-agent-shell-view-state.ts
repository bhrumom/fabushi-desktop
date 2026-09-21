import { useEffect, useState } from 'react';

export interface AgentShellViewStateOptions {
  readonly initialSidebarWidth?: number;
  readonly initialMessageRenderCount: number;
  readonly initialInfoOpen: boolean;
  readonly wideBreakpoint?: number;
}

export function useAgentShellViewState(options: AgentShellViewStateOptions) {
  const [search, setSearch] = useState('');
  const [sidebarWidth, setSidebarWidth] = useState(options.initialSidebarWidth ?? 330);
  const [conversationSearchOpen, setConversationSearchOpen] = useState(false);
  const [agentConversationSearch, setAgentConversationSearch] = useState('');
  const [messageRenderCount, setMessageRenderCount] = useState(options.initialMessageRenderCount);
  const [infoOpen, setInfoOpen] = useState(options.initialInfoOpen);
  const [narrowInfoOpen, setNarrowInfoOpen] = useState(false);
  const [wideInfoLayout, setWideInfoLayout] = useState(
    () => typeof window === 'undefined' ? true : window.innerWidth > (options.wideBreakpoint ?? 1280),
  );
  const [agentSettingsOpen, setAgentSettingsOpen] = useState(false);
  const [showScrollToLatest, setShowScrollToLatest] = useState(false);

  useEffect(() => {
    const breakpoint = options.wideBreakpoint ?? 1280;
    const onResize = () => {
      const nextWide = window.innerWidth > breakpoint;
      setWideInfoLayout(nextWide);
      if (nextWide) setNarrowInfoOpen(false);
    };
    onResize();
    window.addEventListener('resize', onResize);
    return () => window.removeEventListener('resize', onResize);
  }, [options.wideBreakpoint]);

  return {
    search, setSearch,
    sidebarWidth, setSidebarWidth,
    conversationSearchOpen, setConversationSearchOpen,
    agentConversationSearch, setAgentConversationSearch,
    messageRenderCount, setMessageRenderCount,
    infoOpen, setInfoOpen,
    narrowInfoOpen, setNarrowInfoOpen,
    wideInfoLayout, setWideInfoLayout,
    agentSettingsOpen, setAgentSettingsOpen,
    showScrollToLatest, setShowScrollToLatest,
  };
}
