import { AppWindow } from 'lucide-react';
import React from 'react';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import { FabSurface } from '../../ui/primitives/fab-primitives';

export default function MiniAppCompatibilityAdapter({ onClose }: { readonly onClose(): void }) {
  return <CompatibilityFeatureFrame
    title="Mini Apps"
    description="Mini App install, WebMCP and Bot projection stay in their own capability surface."
    onClose={onClose}
  >
    <FabSurface elevated>
      <div style={{ padding: 16, display: 'flex', gap: 10, alignItems: 'center' }}>
        <AppWindow size={20} />
        <div><strong>Marketplace compatibility</strong><p>Mini Apps no longer participate in Agent shell state or Agent identity inference.</p></div>
      </div>
    </FabSurface>
  </CompatibilityFeatureFrame>;
}
