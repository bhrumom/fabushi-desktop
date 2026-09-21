import React from 'react';
import { createPortal } from 'react-dom';
import FabAvatarImpl from '../avatar/fab-avatar';
import type { FabAvatarProps } from '../avatar/fab-avatar';
import styles from './fab-primitives.module.css';

function cx(...values: Array<string | false | null | undefined>): string {
  return values.filter(Boolean).join(' ');
}

export function FabButton({
  variant = 'default',
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & {
  readonly variant?: 'default' | 'primary' | 'danger' | 'ghost';
}) {
  return <button
    {...props}
    className={cx(styles.button, className)}
    data-variant={variant}
  />;
}

export function FabIconButton({
  label,
  size = 'md',
  className,
  children,
  ...props
}: Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'aria-label'> & {
  readonly label: string;
  readonly size?: 'sm' | 'md' | 'lg';
}) {
  return <button
    {...props}
    type={props.type ?? 'button'}
    aria-label={label}
    title={props.title ?? label}
    className={cx(styles.iconButton, className)}
    data-size={size}
  >{children}</button>;
}

export function FabInput({ className, ...props }: React.InputHTMLAttributes<HTMLInputElement>) {
  return <input {...props} className={cx(styles.input, className)} />;
}

export function FabSelect({ className, children, ...props }: React.SelectHTMLAttributes<HTMLSelectElement>) {
  return <select {...props} className={cx(styles.select, className)}>{children}</select>;
}

export function FabBadge({
  tone = 'neutral',
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLSpanElement> & {
  readonly tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger';
}) {
  return <span {...props} className={cx(styles.badge, className)} data-tone={tone}>{children}</span>;
}

export function FabSpinner({
  size = 16,
  label = 'Loading',
  className,
}: {
  readonly size?: number;
  readonly label?: string;
  readonly className?: string;
}) {
  return <span
    className={cx(styles.spinner, className)}
    style={{ '--fab-spinner-size': `${Math.max(10, Math.round(size))}px` } as React.CSSProperties}
    role="status"
    aria-label={label}
  />;
}

export function FabSurface({
  level = 'raised',
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & {
  readonly level?: 'primary' | 'raised' | 'floating';
}) {
  return <div {...props} className={cx(styles.surface, className)} data-level={level} />;
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
  return <span className={cx(styles.tooltip, className)} data-tooltip={label}>{children}</span>;
}

export function FabPopover({
  open,
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & {
  readonly open: boolean;
}) {
  if (!open) return null;
  return <div {...props} className={cx(styles.popover, className)} data-open="true">{children}</div>;
}

export function FabMenu({ className, children, ...props }: React.HTMLAttributes<HTMLDivElement>) {
  return <div {...props} role={props.role ?? 'menu'} className={cx(styles.menu, className)}>{children}</div>;
}

export function FabMenuItem({
  danger = false,
  className,
  ...props
}: React.ButtonHTMLAttributes<HTMLButtonElement> & { readonly danger?: boolean }) {
  return <button
    {...props}
    type={props.type ?? 'button'}
    role={props.role ?? 'menuitem'}
    className={cx(styles.menuItem, className)}
    data-danger={danger || undefined}
  />;
}

export function FabDialog({
  label,
  onClose,
  onSubmit,
  children,
  className,
}: {
  readonly label: string;
  readonly onClose: () => void;
  readonly onSubmit?: React.FormEventHandler<HTMLFormElement>;
  readonly children: React.ReactNode;
  readonly className?: string;
}) {
  const surface = onSubmit
    ? <form className={cx(styles.dialogSurface, className)} onSubmit={onSubmit}>{children}</form>
    : <div className={cx(styles.dialogSurface, className)}>{children}</div>;
  const dialog = <div
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
  // Dialogs must escape Sidebar/Workspace stacking contexts so the overlay
  // is also the top-most hit-test surface, not merely visually present.
  return typeof document === 'undefined' ? dialog : createPortal(dialog, document.body);
}

export function FabDialogActions({ children, className }: { readonly children: React.ReactNode; readonly className?: string }) {
  return <div className={cx(styles.dialogActions, className)}>{children}</div>;
}

export function FabAvatar(props: FabAvatarProps) {
  return <FabAvatarImpl {...props} />;
}

export type { FabAvatarProps };
