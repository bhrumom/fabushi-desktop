import React from 'react';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';

export default function CallsCompatibilityAdapter({ onClose }: { readonly onClose(): void }) {
  return <CompatibilityFeatureFrame
    title="Calls"
    description="Voice/video call state is a messaging feature, not an Agent runtime concern."
    onClose={onClose}
  >
    <p>WebRTC call sessions are intentionally outside ConversationActor and Agent workspace state.</p>
  </CompatibilityFeatureFrame>;
}
