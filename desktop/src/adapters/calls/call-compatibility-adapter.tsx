import React from 'react';
import type { CompatibilitySection } from '../../agent-workspace/messenger-compatibility-adapter';

export default function CallCompatibilityAdapter({
  section,
  children,
}: {
  readonly section: CompatibilitySection;
  readonly children: React.ReactNode;
}) {
  return section === 'calls' ? <>{children}</> : null;
}
