import React from 'react';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';

export default function TelegramCompatibilityAdapter({ onClose }: { readonly onClose(): void }) {
  return <CompatibilityFeatureFrame
    title="Telegram"
    description="Telegram transport remains a compatibility capability rather than an Agent shell owner."
    onClose={onClose}
  >
    <p>Telegram peers and updates are projected through the messaging gateway and cannot create Agent runtime state.</p>
  </CompatibilityFeatureFrame>;
}
