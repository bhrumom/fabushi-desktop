import React from 'react';
import FabAvatar from '../avatar/fab-avatar';
import styles from './fab-primitives.module.css';

export { FabAvatar };

export type FabButtonVariant = 'default' | 'primary' | 'danger' | 'ghost';

export function FabButton({
  variant = 'default',
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  readonly variant?: FabButtonVariant;
}) {
  return <button
    {...props}
    className={[styles.button, className].filter(Boolean).join(' ')}
    data-variant={variant}
  />;
}

export function FabIconButton({
  label,
  variant = 'ghost',
  className,
  children,
  ...props
}: Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'aria-label'> & {
  readonly label: string;
  readonly variant?: FabButtonVariant;
}) {
  return <button
    {...props}
    type={props.type ?? 'button'}
    aria-label={label}
    title={props.title ?? label}
    className={[styles.iconButton, className].filter(Boolean).join(' ')}
    data-variant={variant}
  >{children}</button>;
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

export function FabSelect({
  className,
  ...props
}: React.SelectHTMLAttributes<HTMLSelectElement>) {
  return <select
    {...props}
    className={[styles.select, className].filter(Boolean).join(' ')}
  />;
}

export function FabSurface({
  elevated = false,
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & { readonly elevated?: boolean }) {
  return <div
    {...props}
    className={[styles.surface, className].filter(Boolean).join(' ')}
    data-elevated={elevated || undefined}
  />;
}

export function FabBadge({
  tone = 'neutral',
  className,
  ...props
}: React.HTMLAttributes<HTMLSpanElement> & {
  readonly tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger';
}) {
  return <span
    {...props}
    className={[styles.badge, className].filter(Boolean).join(' ')}
    data-tone={tone}
  />;
}

export function FabSpinner({
  label = 'Loading',
  size = 'md',
}: {
  readonly label?: string;
  readonly size?: 'sm' | 'md' | 'lg';
}) {
  return <span className={styles.spinner} data-size={size} role="status" aria-label={label} />;
}

export function FabTooltip({
  label,
  children,
  className,
}: {
  readonly label: string;
  readonly children: React.ReactNode;
  readonly className?: string;
}) {
  return <span className={[styles.tooltip, className].filter(Boolean).join(' ')} data-tooltip={label}>
    {children}
    <span className={styles.tooltipBubble} role="tooltip">{label}</span>
  </span>;
}

export function FabPopover({
  label,
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & {
  readonly label: string;
}) {
  return <div
    {...props}
    role="dialog"
    aria-label={label}
    className={[styles.popover, className].filter(Boolean).join(' ')}
  >{children}</div>;
}

export function FabMenu({
  label,
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & {
  readonly label: string;
}) {
  return <div
    {...props}
    role="menu"
    aria-label={label}
    className={[styles.menu, className].filter(Boolean).join(' ')}
  >{children}</div>;
}

export function FabDialog({
  label,
  onClose,
  onSubmit,
  className,
  children,
}: {
  readonly label: string;
  readonly onClose: () => void;
  readonly onSubmit?: React.FormEventHandler<HTMLFormElement>;
  readonly className?: string;
  readonly children: React.ReactNode;
}) {
  const surfaceClass = [styles.dialogSurface, className].filter(Boolean).join(' ');
  const surface = onSubmit
    ? <form className={surfaceClass} onSubmit={onSubmit}>{children}</form>
    : <div className={surfaceClass}>{children}</div>;
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
