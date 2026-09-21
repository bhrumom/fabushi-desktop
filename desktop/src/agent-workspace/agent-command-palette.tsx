import React from 'react';
import GrokCommandPalette from '../grok-shell/grok-command-palette';

export type AgentCommandPaletteProps = React.ComponentProps<typeof GrokCommandPalette>;

/**
 * Agent-owned global command surface.
 *
 * The recovered Grok component remains the visual implementation while the
 * primary desktop shell consumes this product-owned boundary. This keeps the
 * navigation/search contract in agent-workspace and lets the presentation
 * implementation evolve without re-coupling Messenger to grok-shell.
 */
export default function AgentCommandPalette(props: AgentCommandPaletteProps) {
  return <GrokCommandPalette {...props} />;
}
