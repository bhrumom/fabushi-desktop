import React from 'react';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';

export default function ContactsCompatibilityAdapter({ onClose }: { readonly onClose(): void }) {
  return <CompatibilityFeatureFrame
    title="Contacts"
    description="Human contact identity and social messaging compatibility live outside the Agent runtime."
    onClose={onClose}
  >
    <p>Contact conversations are isolated from Agent identity, drafts, runs, permissions and sidebar state.</p>
  </CompatibilityFeatureFrame>;
}
