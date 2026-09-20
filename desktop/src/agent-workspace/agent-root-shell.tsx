import React from 'react';

export type AgentRootShellProps = React.ComponentPropsWithoutRef<'main'>;

/**
 * Primary desktop product boundary.
 *
 * The legacy Messenger module may still provide compatibility adapters and
 * secondary overlays, but it no longer owns the root product identity. New
 * Agent-first layout/controller work should enter through this boundary.
 */
export default function AgentRootShell({ children, ...props }: AgentRootShellProps) {
  return <main {...props} data-agent-root-shell="true">{children}</main>;
}
