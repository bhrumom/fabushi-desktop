import React from 'react';
import CompatibilityFeatureFrame from '../compatibility/compatibility-feature-frame';

export default function PaymentsCompatibilityAdapter({ onClose }: { readonly onClose(): void }) {
  return <CompatibilityFeatureFrame
    title="Payments"
    description="Invoices, wallet and entitlement compatibility are isolated from Agent execution."
    onClose={onClose}
  >
    <p>Payment state cannot influence Agent run lifecycle or capability policy.</p>
  </CompatibilityFeatureFrame>;
}
