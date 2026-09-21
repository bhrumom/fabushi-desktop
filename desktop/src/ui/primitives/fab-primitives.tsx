import React from 'react';
import styles from './fab-primitives.module.css';

export function FabButton({
  variant = 'default',
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  readonly variant?: 'default' | 'primary' | 'danger' | 'ghost';
}) {
  return <button
    {...props}
    className={[styles.button, className].filter(Boolean).join(' ')}
    data-variant={variant}
  />;
}

export function FabInput({
  className,
  ...props
}: React.InputHTMLAttributes<HTMLInputElement>) {
  return <input
    {...props}
    className={[styles.input, className].filter(Boolean).join(' ')}
  />;
}

export function FabDialog({
  label,
  onClose,
  onSubmit,
  children,
}: {
  readonly label: string;
  readonly onClose: () => void;
  readonly onSubmit?: React.FormEventHandler<HTMLFormElement>;
  readonly children: React.ReactNode;
}) {
  const surface = onSubmit
    ? <form className={styles.dialogSurface} onSubmit={onSubmit}>{children}</form>
    : <div className={styles.dialogSurface}>{children}</div>;
  return <div
    role="dialog"
    aria-modal="true"
    aria-label={label}
    className={styles.dialogOverlay}
    onMouseDown={(event) => {
      if (event.target === event.currentTarget) onClose();
    }}
  >
    {surface}
  </div>;
}

export function FabDialogActions({ children }: { readonly children: React.ReactNode }) {
  return <div className={styles.dialogActions}>{children}</div>;
}