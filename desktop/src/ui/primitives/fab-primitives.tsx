import React from 'react';
import styles from './fab-primitives.module.css';

export { default as FabAvatar } from '../avatar/fab-avatar';
export type { FabAvatarInputState, FabAvatarProps, FabAvatarState } from '../avatar/fab-avatar';

function classes(...values: Array<string | false | null | undefined>): string {
  return values.filter(Boolean).join(' ');
}

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
    className={classes(styles.button, className)}
    data-variant={variant}
  />;
}

export function FabIconButton({
  label,
  size = 'default',
  variant = 'ghost',
  className,
  children,
  ...props
}: Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'aria-label'> & {
  readonly label: string;
  readonly size?: 'small' | 'default' | 'large';
  readonly variant?: FabButtonVariant;
}) {
  return <button
    {...props}
    type={props.type ?? 'button'}
    aria-label={label}
    title={props.title ?? label}
    className={classes(styles.button, styles.iconButton, className)}
    data-size={size}
    data-variant={variant}
  >{children}</button>;
}

export function FabInput({
  className,
  ...props
}: React.InputHTMLAttributes<HTMLInputElement>) {
  return <input
    {...props}
    className={classes(styles.input, className)}
  />;
}

export function FabSelect({
  className,
  ...props
}: React.SelectHTMLAttributes<HTMLSelectElement>) {
  return <select
    {...props}
    className={classes(styles.input, styles.select, className)}
  />;
}

export function FabSurface({
  elevation = 'base',
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & {
  readonly elevation?: 'base' | 'raised' | 'floating';
}) {
  return <div
    {...props}
    className={classes(styles.surface, className)}
    data-elevation={elevation}
  />;
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
    className={classes(styles.menu, className)}
  >{children}</div>;
}

export function FabPopover({
  label,
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLDivElement> & {
  readonly label?: string;
}) {
  return <div
    {...props}
    role={label ? 'dialog' : props.role}
    aria-label={label}
    className={classes(styles.popover, className)}
  >{children}</div>;
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
  return <span
    className={classes(styles.tooltip, className)}
    data-tooltip={label}
  >{children}</span>;
}

export function FabBadge({
  tone = 'neutral',
  className,
  children,
  ...props
}: React.HTMLAttributes<HTMLSpanElement> & {
  readonly tone?: 'neutral' | 'accent' | 'success' | 'warning' | 'danger';
}) {
  return <span
    {...props}
    className={classes(styles.badge, className)}
    data-tone={tone}
  >{children}</span>;
}

export function FabSpinner({
  label = 'Loading',
  size = 'default',
  className,
}: {
  readonly label?: string;
  readonly size?: 'small' | 'default' | 'large';
  readonly className?: string;
}) {
  return <span
    role="status"
    aria-label={label}
    className={classes(styles.spinner, className)}
    data-size={size}
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
