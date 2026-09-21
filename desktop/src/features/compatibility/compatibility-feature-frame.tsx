import React from 'react';
import { FabButton, FabSurface } from '../../ui/primitives/fab-primitives';
import styles from './compatibility-feature-frame.module.css';

export default function CompatibilityFeatureFrame({
  title,
  description,
  onClose,
  children,
}: {
  readonly title: string;
  readonly description: string;
  readonly onClose: () => void;
  readonly children?: React.ReactNode;
}) {
  return <section className={styles.root} data-compatibility-feature="true">
    <header>
      <div><strong>{title}</strong><p>{description}</p></div>
      <FabButton type="button" variant="ghost" onClick={onClose}>Back to Agents</FabButton>
    </header>
    <FabSurface className={styles.body} elevated>{children}</FabSurface>
  </section>;
}
