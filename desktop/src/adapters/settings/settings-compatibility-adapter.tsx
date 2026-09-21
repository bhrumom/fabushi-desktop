import React from 'react';
import type { CompatibilitySection } from '../../agent-workspace/messenger-compatibility-adapter';

export default function SettingsCompatibilityAdapter({
  section,
  children,
}: {
  readonly section: CompatibilitySection;
  readonly children: React.ReactNode;
}) {
  return section === 'settings' ? <>{children}</> : null;
}
