import React from 'react';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';
import frameStyles from '../compatibility/compatibility-feature-frame.module.css';
import { FabButton, FabSurface } from '../../ui/primitives/fab-primitives';

export default function SettingsCompatibilityAdapter({
  onClose,
  onLogout,
}: {
  readonly onClose(): void;
  readonly onLogout(): Promise<void>;
}) {
  return <CompatibilityFeatureFrame
    title="Settings"
    description="Account/product settings are isolated from per-Agent runtime ownership."
    onClose={onClose}
  >
    <FabSurface elevated className={frameStyles.stack}>
        <strong>Account</strong>
        <p>Signing out clears the authenticated desktop session without mutating Agent controller ownership.</p>
        <div className={frameStyles.actions}><FabButton type="button" variant="danger" onClick={() => void onLogout()}>Sign out</FabButton></div>
    </FabSurface>
  </CompatibilityFeatureFrame>;
}
